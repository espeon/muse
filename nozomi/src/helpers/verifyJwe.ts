import { jwtDecrypt, JWTPayload } from "jose";
import { hkdf } from "@panva/hkdf";

export async function verifyJWE(
  token: string,
  cookieName: string,
): Promise<JWTPayload> {
  let secret = process.env.AUTH_SECRET;
  if (!secret) {
    throw new Error("AUTH_SECRET is not set in the environment");
  }
  const payload = await decryptJWT(token, secret, cookieName);
  console.log(payload, Date.now());
  // is payload expired?
  if (
    !payload ||
    (payload.exp && payload.exp < Date.now() / 1000) ||
    (payload.iat && payload.iat > Date.now() / 1000)
  ) {
    throw new Error("JWT has expired");
  }
  return payload;
}

async function decryptJWT(
  token: string | Uint8Array,
  secret: string,
  salt: string,
) {
  try {
    const { payload } = await jwtDecrypt(
      token,
      async ({ enc }) => {
        const encryptionSecret = await getDerivedEncryptionKey(
          enc,
          secret,
          salt,
        );
        return encryptionSecret;
      },
      {
        clockTolerance: 15,
        keyManagementAlgorithms: ["dir"],
        contentEncryptionAlgorithms: ["A256CBC-HS512", "A256GCM"],
      },
    );
    return payload;
  } catch (error) {
    console.error("Failed to decrypt JWT:", error);
    return null;
  }
}

async function getDerivedEncryptionKey(
  enc: string,
  keyMaterial: string,
  salt: string,
) {
  let length;
  switch (enc) {
    case "A256CBC-HS512":
      length = 64;
      break;
    case "A256GCM":
      length = 32;
      break;
    default:
      throw new Error("Unsupported JWT Content Encryption Algorithm");
  }
  return await hkdf(
    "sha256",
    keyMaterial,
    salt,
    `Auth.js Generated Encryption Key (${salt})`,
    length,
  );
}
