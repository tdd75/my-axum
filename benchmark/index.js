import http from "k6/http";
import { check, sleep } from "k6";
import {
  randomIntBetween,
  randomItem,
} from "https://jslib.k6.io/k6-utils/1.2.0/index.js";

const BASE_URL = "http://0.0.0.0:8000/api/v1";

export const options = {
  stages: [
    { duration: "1m", target: 1000 }, // Ramp up to 50 users
    // { duration: "2m", target: 100 }, // Stay at 100 users
    // { duration: "1m", target: 200 }, // Spike to 200 users
    // { duration: "1m", target: 0 }, // Ramp down
  ],
  thresholds: {
    http_req_duration: ["p(95)<500"], // 95% of requests should be below 500ms
    http_req_failed: ["rate<0.05"], // Error rate should be below 5%
  },
};

// Shared data
let globalToken = null;
let createdUserIds = [];

export function setup() {
  // Try to login first with default credentials
  const loginPayload = JSON.stringify({
    email: "admin@example.com",
    password: "admin123@",
  });

  const loginRes = http.post(`${BASE_URL}/auth/login/`, loginPayload, {
    headers: { "Content-Type": "application/json" },
  });

  console.log(`Login status: ${loginRes.status}`);

  if (loginRes.status === 200) {
    try {
      const body = JSON.parse(loginRes.body);
      if (body && body.access) {
        console.log("Successfully got token from login");
        return { token: body.access };
      }
      console.log(`Login failed - response: ${JSON.stringify(body)}`);
    } catch (e) {
      console.error(`Failed to parse login response: ${e}`);
    }
  }

  // Fallback: try to register
  const registerPayload = JSON.stringify({
    email: `loadtest_${Date.now()}@example.com`,
    password: "Password123!",
    first_name: "Load",
    last_name: "Test",
  });

  const registerRes = http.post(`${BASE_URL}/auth/register/`, registerPayload, {
    headers: { "Content-Type": "application/json" },
  });

  console.log(`Register status: ${registerRes.status}`);

  if (registerRes.status === 200 || registerRes.status === 201) {
    try {
      const body = JSON.parse(registerRes.body);
      if (body && body.access) {
        console.log("Successfully got token from register");
        return { token: body.access };
      }
      console.log(`Register failed - response: ${JSON.stringify(body)}`);
    } catch (e) {
      console.error(`Failed to parse register response: ${e}`);
    }
  }

  console.error("Failed to get token from both login and register");
  return { token: null };
}

export default function (data) {
  // Refresh token every iteration to avoid expiration (use shared token or login)
  let token = data.token;

  // Randomly login to get fresh token (1% of requests)
  if (Math.random() < 0.01) {
    const loginPayload = JSON.stringify({
      email: "admin@example.com",
      password: "admin123@",
    });

    const loginRes = http.post(`${BASE_URL}/auth/login/`, loginPayload, {
      headers: { "Content-Type": "application/json" },
    });

    if (loginRes.status === 200) {
      try {
        const body = JSON.parse(loginRes.body);
        if (body && body.access) {
          token = body.access; // Use fresh token for this iteration
        }
      } catch (e) {
        // Keep using old token
      }
    }
  }

  if (!token) {
    console.error("No valid token available");
    return;
  }

  const headers = {
    "Content-Type": "application/json",
    Authorization: `Bearer ${token}`,
  };

  // Randomly choose an action - weighted towards read operations
  const actions = [
    "searchUsers",
    "searchUsers",
    "searchUsers",
    "getMe",
    "getMe",
    "getMe",
    "getMe",
    "getMe",
    "getUser",
    "getUser",
    "getUser",
    "getUser",
    "createUser",
    "createUser",
    "updateUser",
    "updateUser",
    "updateUser",
  ];

  const action = randomItem(actions);

  switch (action) {
    case "searchUsers":
      searchUsers(headers);
      break;
    case "getMe":
      getMe(headers);
      break;
    case "createUser":
      createUser(headers);
      break;
    case "getUser":
      getUser(headers);
      break;
    case "updateUser":
      updateUser(headers);
      break;
  }

  sleep(randomIntBetween(0.5, 1)); // Minimal sleep for high throughput
}

function searchUsers(headers) {
  const page = randomIntBetween(1, 5);
  const pageSize = randomIntBetween(5, 20);

  const res = http.get(`${BASE_URL}/user/?page=${page}&page_size=${pageSize}`, {
    headers,
  });

  check(res, {
    "search users - status 200": (r) => r.status === 200,
    "search users - has data": (r) => {
      try {
        const body = JSON.parse(r.body);
        return body && Array.isArray(body.items);
      } catch {
        return false;
      }
    },
  });
}

function getMe(headers) {
  const res = http.get(`${BASE_URL}/auth/me/`, { headers });

  check(res, {
    "get me - status 200": (r) => r.status === 200,
    "get me - has user data": (r) => {
      try {
        const body = JSON.parse(r.body);
        return body && body.id;
      } catch {
        return false;
      }
    },
  });
}

function createUser(headers) {
  const timestamp = Date.now();
  const payload = JSON.stringify({
    email: `user_${timestamp}_${randomIntBetween(1, 10000)}@example.com`,
    password: "Password123!",
    first_name: `FirstName${randomIntBetween(1, 1000)}`,
    last_name: `LastName${randomIntBetween(1, 1000)}`,
  });

  const res = http.post(`${BASE_URL}/user/`, payload, { headers });

  check(res, {
    "create user - status 200 or 201": (r) =>
      r.status === 200 || r.status === 201,
    "create user - has id": (r) => {
      try {
        const body = JSON.parse(r.body);
        if (body && body.id) {
          createdUserIds.push(body.id);
          return true;
        }
        return false;
      } catch {
        return false;
      }
    },
  });
}

function getUser(headers) {
  // Use a random user ID (assuming users exist with IDs 1-100)
  const userId =
    createdUserIds.length > 0
      ? randomItem(createdUserIds)
      : randomIntBetween(1, 100);

  const res = http.get(`${BASE_URL}/user/${userId}/`, { headers });

  check(res, {
    "get user - status 200 or 404": (r) => r.status === 200 || r.status === 404,
  });
}

function updateUser(headers) {
  const userId =
    createdUserIds.length > 0
      ? randomItem(createdUserIds)
      : randomIntBetween(1, 100);

  const payload = JSON.stringify({
    first_name: `Updated${randomIntBetween(1, 1000)}`,
    last_name: `User${randomIntBetween(1, 1000)}`,
  });

  const res = http.patch(`${BASE_URL}/user/${userId}/`, payload, { headers });

  check(res, {
    "update user - status 200 or 404": (r) =>
      r.status === 200 || r.status === 404,
  });
}

export function teardown(data) {
  console.log("Load test completed");
  console.log(`Total users created: ${createdUserIds.length}`);
}
