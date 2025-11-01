import http from 'k6/http';
import { check, fail, group, sleep } from 'k6';

const BASE_URL = (__ENV.BASE_URL || 'http://localhost:8000').replace(/\/+$/, '');
const RUN_ID = __ENV.TEST_RUN_ID || String(Date.now());

const TARGET_VUS = numberEnv('TARGET_VUS', 1000);
const START_VUS = numberEnv('START_VUS', Math.min(50, TARGET_VUS));
const STAGE_1_TARGET = numberEnv(
    'STAGE_1_TARGET',
    Math.max(START_VUS, Math.min(TARGET_VUS, 250)),
);
const STAGE_2_TARGET = numberEnv(
    'STAGE_2_TARGET',
    Math.max(STAGE_1_TARGET, Math.min(TARGET_VUS, 600)),
);

const THINK_TIME_MIN = numberEnv('THINK_TIME_MIN', 0.5);
const THINK_TIME_MAX = numberEnv('THINK_TIME_MAX', 2);
const REQUEST_TIMEOUT = __ENV.REQUEST_TIMEOUT || '30s';
const PASSWORD = __ENV.BENCHMARK_PASSWORD || 'Benchmark123@';
const ENABLE_USER_CRUD = boolEnv('ENABLE_USER_CRUD', true);
const DEBUG = boolEnv('DEBUG_BENCH', false);

const ADMIN_EMAIL = __ENV.ADMIN_EMAIL || 'admin@example.com';
const ADMIN_PASSWORD = __ENV.ADMIN_PASSWORD || 'admin123@';

export const options = {
    scenarios: {
        mixed_api_traffic: {
            executor: 'ramping-vus',
            startVUs: START_VUS,
            gracefulRampDown: '30s',
            stages: [
                { duration: __ENV.RAMP_STAGE_1 || '30s', target: STAGE_1_TARGET },
                { duration: __ENV.RAMP_STAGE_2 || '30s', target: STAGE_2_TARGET },
                { duration: __ENV.RAMP_STAGE_3 || '1m', target: TARGET_VUS },
                { duration: __ENV.HOLD_STAGE || '2m', target: TARGET_VUS },
                { duration: __ENV.RAMP_DOWN_STAGE || '1m', target: 0 },
            ],
        },
    },
    thresholds: {
        http_req_failed: ['rate<0.05'],
        checks: ['rate>0.99'],
        'http_req_duration{kind:read}': ['p(95)<800'],
        'http_req_duration{kind:write}': ['p(95)<1500'],
        'http_req_duration{kind:auth}': ['p(95)<1200'],
    },
    summaryTrendStats: ['avg', 'min', 'med', 'p(90)', 'p(95)', 'p(99)', 'max'],
};

let userSession = null;
let adminSession = null;
let knownUserIds = [];
let managedUserIds = [];
let identityCounter = 0;
let managedCounter = 0;

export function setup() {
    const res = http.get(
        `${BASE_URL}/docs/openapi.json`,
        requestParams(null, 'openapi_smoke', 'read'),
    );

    if (res.status !== 200) {
        throw new Error(
            `Cannot reach ${BASE_URL}. Expected /docs/openapi.json to return 200, got ${res.status}`,
        );
    }

    verifyAdminAccess();
}

export default function () {
    ensureUserSession();

    const roll = Math.random() * 100;

    if (roll < 22) {
        group('auth.get_profile', getProfileFlow);
    } else if (roll < 42) {
        group('user.search', searchUsersFlow);
    } else if (roll < 56) {
        group('user.get_detail', getUserDetailFlow);
    } else if (roll < 66) {
        group('auth.refresh_token', refreshTokenFlow);
    } else if (roll < 76) {
        group('auth.update_profile', updateProfileFlow);
    } else if (roll < 84) {
        group('user.create', createUserFlow);
    } else if (roll < 90) {
        group('user.update', updateUserFlow);
    } else if (roll < 93) {
        group('user.delete', deleteUserFlow);
    } else {
        group('auth.logout_login', logoutLoginFlow);
    }

    sleep(randomBetween(THINK_TIME_MIN, THINK_TIME_MAX));
}

function getProfileFlow() {
    const session = ensureUserSession();
    const res = http.get(
        `${BASE_URL}/api/v1/auth/me/`,
        requestParams(session, 'auth_me', 'read'),
    );

    if (recoverAuth(session, res)) {
        return;
    }

    const body = safeJson(res);

    check(res, {
        'auth/me returns 200': (r) => r.status === 200,
        'auth/me returns user id': () => body && Number(body.id) > 0,
    });

    if (body && body.id) {
        session.userId = body.id;
        rememberUserId(body.id);
    }
}

function updateProfileFlow() {
    const session = ensureUserSession();
    const suffix = `${__VU}-${__ITER}`;
    const payload = {
        first_name: `Bench-${suffix}`,
        phone: `09${String(__VU).padStart(4, '0')}${String(__ITER % 10000).padStart(4, '0')}`,
    };

    const res = http.patch(
        `${BASE_URL}/api/v1/auth/me/`,
        JSON.stringify(payload),
        requestParams(session, 'auth_update_profile', 'write'),
    );

    if (recoverAuth(session, res)) {
        return;
    }

    const body = safeJson(res);

    check(res, {
        'auth/me patch returns 200': (r) => r.status === 200,
        'auth/me patch echoes first_name': () =>
            body && body.first_name === payload.first_name,
    });
}

function refreshTokenFlow() {
    const session = ensureUserSession();
    const res = http.post(
        `${BASE_URL}/api/v1/auth/refresh-token/`,
        JSON.stringify({ refresh_token: session.refreshToken }),
        requestParams(null, 'auth_refresh_token', 'auth'),
    );

    if (res.status === 401) {
        userSession = null;
        ensureUserSession();
        return;
    }

    const body = safeJson(res);

    check(res, {
        'refresh-token returns 200': (r) => r.status === 200,
        'refresh-token returns access token': () => body && !!body.access,
        'refresh-token rotates refresh token': () =>
            body && !!body.refresh && body.refresh !== session.refreshToken,
    });

    if (body && body.access && body.refresh) {
        session.accessToken = body.access;
        session.refreshToken = body.refresh;
    }
}

function logoutLoginFlow() {
    const session = ensureUserSession();
    const logoutRes = http.request(
        'POST',
        `${BASE_URL}/api/v1/auth/logout/`,
        null,
        requestParams(session, 'auth_logout', 'auth', {
            cookies: { refresh_token: session.refreshToken },
        }),
    );

    if (recoverAuth(session, logoutRes)) {
        return;
    }

    check(logoutRes, {
        'logout returns 204': (r) => r.status === 204,
    });

    userSession = loginWithCredentials(session.email, session.password, 'vu-user');
}

function searchUsersFlow() {
    const session = ensureAdminSession();
    const ownPrefix = ownEmailPrefix();
    const queries = [
        `/api/v1/user/?page=1&page_size=10&order_by=-created_at`,
        `/api/v1/user/?page=1&page_size=5&order_by=+first_name`,
        `/api/v1/user/?email=${encodeURIComponent(ownPrefix)}&page=1&page_size=5`,
    ];

    const res = http.get(
        `${BASE_URL}${pickRandom(queries)}`,
        requestParams(session, 'user_search', 'read'),
    );

    if (recoverAuth(session, res)) {
        return;
    }

    const body = safeJson(res);
    const items = body && Array.isArray(body.items) ? body.items : [];

    check(res, {
        'user search returns 200': (r) => r.status === 200,
        'user search returns items array': () => Array.isArray(items),
    });
}

function getUserDetailFlow() {
    const session = ensureAdminSession();
    const userId = pickKnownUserId();

    if (!userId) {
        getProfileFlow();
        return;
    }

    const res = http.get(
        `${BASE_URL}/api/v1/user/${userId}/`,
        requestParams(session, 'user_get_detail', 'read'),
    );

    if (recoverAuth(session, res)) {
        return;
    }

    const body = safeJson(res);

    check(res, {
        'user detail returns 200': (r) => r.status === 200,
        'user detail returns expected id': () => body && Number(body.id) === Number(userId),
    });

    if (body && body.id) {
        rememberUserId(body.id);
    }
}

function createUserFlow() {
    if (!ENABLE_USER_CRUD) {
        searchUsersFlow();
        return;
    }

    const session = ensureAdminSession();
    const email = nextManagedEmail();
    const payload = {
        email: email,
        password: PASSWORD,
        first_name: `Managed-${__VU}`,
        last_name: `User-${managedCounter}`,
        phone: `08${String(__VU).padStart(4, '0')}${String(managedCounter % 10000).padStart(4, '0')}`,
    };

    const res = http.post(
        `${BASE_URL}/api/v1/user/`,
        JSON.stringify(payload),
        requestParams(session, 'user_create', 'write'),
    );

    if (recoverAuth(session, res)) {
        return;
    }

    const body = safeJson(res);

    check(res, {
        'user create returns 201': (r) => r.status === 201,
        'user create returns new id': () => body && Number(body.id) > 0,
    });

    if (body && body.id) {
        managedUserIds.push(body.id);
        rememberUserId(body.id);
    }
}

function updateUserFlow() {
    if (!ENABLE_USER_CRUD) {
        updateProfileFlow();
        return;
    }

    if (!managedUserIds.length) {
        createUserFlow();
    }

    const targetId = pickRandom(managedUserIds);
    if (!targetId) {
        return;
    }

    const session = ensureAdminSession();
    const payload = {
        first_name: `Updated-${__VU}-${__ITER}`,
        last_name: `Managed-${targetId}`,
    };

    const res = http.patch(
        `${BASE_URL}/api/v1/user/${targetId}/`,
        JSON.stringify(payload),
        requestParams(session, 'user_update', 'write'),
    );

    if (recoverAuth(session, res)) {
        return;
    }

    const body = safeJson(res);

    check(res, {
        'user update returns 200': (r) => r.status === 200,
        'user update echoes target id': () => body && Number(body.id) === Number(targetId),
    });
}

function deleteUserFlow() {
    if (!ENABLE_USER_CRUD || !managedUserIds.length) {
        return;
    }

    const session = ensureAdminSession();
    const targetId = pickRandom(managedUserIds);

    if (!targetId) {
        return;
    }

    const res = http.del(
        `${BASE_URL}/api/v1/user/${targetId}/`,
        null,
        requestParams(session, 'user_delete', 'write'),
    );

    if (recoverAuth(session, res)) {
        return;
    }

    check(res, {
        'user delete returns 204': (r) => r.status === 204,
    });

    managedUserIds = managedUserIds.filter((id) => id !== targetId);
    knownUserIds = knownUserIds.filter((id) => id !== targetId);
}

function ensureUserSession() {
    if (userSession && userSession.accessToken) {
        return userSession;
    }

    userSession = registerVuUser();
    return userSession;
}

function ensureAdminSession() {
    if (adminSession && adminSession.accessToken) {
        return adminSession;
    }

    adminSession = loginWithCredentials(ADMIN_EMAIL, ADMIN_PASSWORD, 'admin');
    return adminSession;
}

function registerVuUser() {
    const email = nextIdentityEmail();
    const payload = {
        email: email,
        password: PASSWORD,
        first_name: `Bench-${__VU}`,
        last_name: `Vu-${__VU}`,
        phone: `07${String(__VU).padStart(4, '0')}${String(identityCounter % 10000).padStart(4, '0')}`,
    };

    const res = http.post(
        `${BASE_URL}/api/v1/auth/register/`,
        JSON.stringify(payload),
        requestParams(null, 'auth_register', 'auth'),
    );

    if (res.status === 409) {
        return loginWithCredentials(email, PASSWORD, 'vu-user');
    }

    const body = safeJson(res);

    const passed = check(res, {
        'register returns 200': (r) => r.status === 200,
        'register returns access token': () => body && !!body.access,
        'register returns refresh token': () => body && !!body.refresh,
    });

    if (!passed || !body || !body.access || !body.refresh) {
        fail(`Bootstrap register failed for ${email} with status ${res.status}`);
    }

    const session = {
        kind: 'user',
        email: email,
        password: PASSWORD,
        accessToken: body.access,
        refreshToken: body.refresh,
        userId: null,
    };

    hydrateSessionProfile(session);
    return session;
}

function loginWithCredentials(email, password, kind) {
    const res = http.post(
        `${BASE_URL}/api/v1/auth/login/`,
        JSON.stringify({ email: email, password: password }),
        requestParams(null, kind === 'admin' ? 'auth_login_admin' : 'auth_login', 'auth'),
    );
    const body = safeJson(res);

    const passed = check(res, {
        'login returns 200': (r) => r.status === 200,
        'login returns access token': () => body && !!body.access,
        'login returns refresh token': () => body && !!body.refresh,
    });

    if (!passed || !body || !body.access || !body.refresh) {
        fail(`Login failed for ${email} with status ${res.status}`);
    }

    const session = {
        kind: kind,
        email: email,
        password: password,
        accessToken: body.access,
        refreshToken: body.refresh,
        userId: null,
    };

    hydrateSessionProfile(session);
    return session;
}

function hydrateSessionProfile(session) {
    const res = http.get(
        `${BASE_URL}/api/v1/auth/me/`,
        requestParams(session, 'auth_me_bootstrap', 'auth'),
    );
    const body = safeJson(res);

    const passed = check(res, {
        'bootstrap me returns 200': (r) => r.status === 200,
        'bootstrap me returns id': () => body && Number(body.id) > 0,
    });

    if (!passed || !body || !body.id) {
        fail(`Failed to hydrate profile for ${session.email} with status ${res.status}`);
    }

    session.userId = body.id;
    rememberUserId(body.id);
}

function recoverAuth(session, res) {
    if (res.status !== 401) {
        return false;
    }

    if (session.kind === 'admin') {
        debugLog('admin session expired, re-login');
        adminSession = null;
        ensureAdminSession();
        return true;
    }

    debugLog('user session expired, re-register');
    userSession = null;
    ensureUserSession();
    return true;
}

function requestParams(session, endpoint, kind, extra) {
    const vu = currentVu();
    const iter = currentIter();
    const headers = {
        Accept: 'application/json',
        'User-Agent': `k6-my-axum/${RUN_ID}`,
        'X-Forwarded-For': `10.${vu % 250}.${(iter + 1) % 250}.1`,
    };

    if (session && session.accessToken) {
        headers.Authorization = `Bearer ${session.accessToken}`;
    }

    if (extra && extra.cookies) {
        const cookieHeader = buildCookieHeader(extra.cookies);
        if (cookieHeader) {
            headers.Cookie = cookieHeader;
        }
    }

    if (!extra || extra.body !== false) {
        headers['Content-Type'] = 'application/json';
    }

    return {
        headers: headers,
        tags: {
            endpoint: endpoint,
            kind: kind,
        },
        timeout: REQUEST_TIMEOUT,
    };
}

function buildCookieHeader(cookies) {
    const pairs = [];
    const names = Object.keys(cookies);

    for (let i = 0; i < names.length; i += 1) {
        const name = names[i];
        const value = cookies[name];

        if (value) {
            pairs.push(`${name}=${value}`);
        }
    }

    return pairs.join('; ');
}

function verifyAdminAccess() {
    const res = http.post(
        `${BASE_URL}/api/v1/auth/login/`,
        JSON.stringify({ email: ADMIN_EMAIL, password: ADMIN_PASSWORD }),
        requestParams(null, 'auth_login_admin_setup', 'auth'),
    );
    const body = safeJson(res);

    if (res.status !== 200 || !body || !body.access || !body.refresh) {
        throw new Error(
            `Admin login failed for ${ADMIN_EMAIL}. Seed data with 'make db-seed' or override ADMIN_EMAIL/ADMIN_PASSWORD before running k6.`,
        );
    }
}

function rememberUserId(id) {
    if (!id || knownUserIds.indexOf(id) !== -1) {
        return;
    }

    knownUserIds.push(id);
}

function pickKnownUserId() {
    if (managedUserIds.length) {
        return pickRandom(managedUserIds);
    }

    if (adminSession && adminSession.userId) {
        return adminSession.userId;
    }

    if (userSession && userSession.userId) {
        return userSession.userId;
    }

    if (knownUserIds.length) {
        return pickRandom(knownUserIds);
    }

    return null;
}

function nextIdentityEmail() {
    identityCounter += 1;
    return `k6-user+${RUN_ID}.vu${currentVu()}.n${identityCounter}@example.com`;
}

function nextManagedEmail() {
    managedCounter += 1;
    return `k6-managed+${RUN_ID}.vu${currentVu()}.n${managedCounter}@example.com`;
}

function ownEmailPrefix() {
    const session = ensureUserSession();
    return session.email.split('@')[0];
}

function safeJson(res) {
    try {
        return res.json();
    } catch (err) {
        debugLog(`json parse failed for ${res.request.method} ${res.request.url}: ${String(err)}`);
        return null;
    }
}

function pickRandom(items) {
    if (!items || !items.length) {
        return null;
    }

    return items[Math.floor(Math.random() * items.length)];
}

function randomBetween(min, max) {
    if (max <= min) {
        return min;
    }

    return Math.random() * (max - min) + min;
}

function numberEnv(name, fallback) {
    const raw = __ENV[name];
    if (!raw) {
        return fallback;
    }

    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : fallback;
}

function boolEnv(name, fallback) {
    const raw = __ENV[name];
    if (raw === undefined) {
        return fallback;
    }

    return ['1', 'true', 'yes', 'on'].indexOf(String(raw).toLowerCase()) !== -1;
}

function currentVu() {
    return typeof __VU === 'number' ? __VU : 0;
}

function currentIter() {
    return typeof __ITER === 'number' ? __ITER : 0;
}

function debugLog(message) {
    if (!DEBUG) {
        return;
    }

    console.log(`[vu=${currentVu()} iter=${currentIter()}] ${message}`);
}
