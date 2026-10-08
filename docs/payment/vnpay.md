# VNPay QR Integration Guide

This document summarizes how the June Pea backend integrates with VNPay's QR checkout flow. It is based on https://sandbox.vnpayment.vn/apis/docs/thanh-toan-pay/pay.html and the sandbox credentials provided by the merchant account (`LINSR88X`).

## Required Environment Variables

Add the following keys to your environment (see `.env` and deployment secrets):

| Variable | Description | Example |
| --- | --- | --- |
| `VNPAY_TMNCODE` | Terminal / Website code | `LINSR88X` |
| `VNPAY_HASH_SECRET` | Shared hash secret used for all signatures | `WF46RHBU0XEMXMVE1ZSJ...` |
| `VNPAY_PAYMENT_URL` | Payment gateway base URL | `https://sandbox.vnpayment.vn/paymentv2/vpcpay.html` |
| `VNPAY_RETURN_URL` | Frontend callback URL shown to customers after payment | `https://localhost:8080/payment/vnpay/return` |
| `VNPAY_IPN_URL` | Public backend endpoint that VNPay calls with payment results | `https://localhost:3000/api/v1/payment/vnpay/ipn` |
| `VNPAY_API_URL` | Merchant API used for refunds (optional, defaults to the sandbox) | `https://sandbox.vnpayment.vn/merchant_webapi/api/transaction` |
| `VNPAY_SERVER_IP` | Our IP as sent in `vnp_IpAddr` on API calls (optional) | `127.0.0.1` |
| `VNPAY_API_TIMEOUT_SECS` | Timeout for API calls (optional) | `10` |
| `VNPAY_DEFAULT_LOCALE` | `vn` or `en` | `vn` |
| `VNPAY_ORDER_TYPE` | Custom category code (defaults to `other`) | `fashion` |

> **Note:** Keep the real hash secret in a secure vault. Only store it locally for testing.

## Flow Overview

1. **Order creation** – The customer submits the checkout form, and we persist an order in the `ordering` service. Orders start with `status = pending`.
2. **Payment intent** – The frontend calls `POST /api/v1/payment/vnpay/qr` with the order ID. The payment service:
   - Validates the order via the ordering usecase.
   - Generates a unique `vnp_TxnRef` and amount in VNPay format.
   - Signs all required parameters using `VNPAY_HASH_SECRET` and stores a `payment.payments` row with status `pending`.
   - Returns the signed checkout URL together with an SVG QR code so the UI can render a modal instantly.
3. **Customer scan** – Customer scans the QR code (or opens the VNPay URL) and authorizes the payment inside the VNPay app.
4. **VNPay IPN** – VNPay calls our IPN endpoint (`/api/v1/payment/vnpay/ipn`) with query parameters describing the transaction. We verify the HMAC signature, match the `vnp_TxnRef`, check amount consistency, and update both the payment record and the original order status.
5. **Frontend polling** – The frontend polls `GET /api/v1/payment/orders/{order_id}` every few seconds to know when the status changes to `paid`, and then shows a success message / clears the cart.

## Refunds

`POST /api/v1/payment/orders/{order_id}/refund` (admin only) asks VNPay for a full refund
(`vnp_Command=refund`, `vnp_TransactionType=02`) and only then marks the payment `refunded`. If
VNPay refuses the answer is `409`, if it is unreachable or answers something we can't verify it
is `502`, and in both cases the payment stays `paid`. VNPay finds the payment by `vnp_TxnRef` and
`vnp_TransactionDate`, which is the `vnp_CreateDate` we put in the payment URL.

Open points: VNPay documents its dates as GMT+7 while the payment URL has always been sent in
UTC (refunds quote the same value back), and if VNPay accepts a refund but saving it fails the
refund exists only in the log. Neither has been checked against the live sandbox.

## Signature Rules

- VNPay expects an HMAC-SHA512 signature over the sorted query string (no URL encoding). The shared secret is used as the HMAC key.
- Requests **to** VNPay include `vnp_SecureHash` and `vnp_SecureHashType=HmacSHA512`.
- IPN callbacks **from** VNPay must be validated by recomputing the signature after removing `vnp_SecureHash` and `vnp_SecureHashType` from the parameter list.
- A successful transaction is indicated by both `vnp_ResponseCode = "00"` and `vnp_TransactionStatus = "00"`.

## Local Testing Checklist

1. Start the stack (`docker compose up postgres redis backend frontend`).
2. Ensure the new migration `07_payment.sql` has been applied (the backend runs migrations on boot).
3. Set env vars (you can base on `.env.example`).
4. Use the web checkout to create an order and click **Thanh toán VNPay**. A QR modal should appear instantly.
5. Approve the transaction from the VNPay sandbox UI (`https://sandbox.vnpayment.vn/merchantv2/`).
6. Observe the backend logs for a `Payment paid` trace and confirm the order status becomes `completed`.
7. Verify `/api/v1/payment/orders/{order_id}` returns `status = paid`.

## Troubleshooting

| Symptom | Action |
| --- | --- |
| VNPay rejects the request with `97` | Signature mismatch – double-check parameter ordering and secret. |
| Payment stays `pending` | Ensure the IPN URL is reachable from the internet (or tunnel via `ngrok`). |
| Amount mismatch (`04`) | Remember VNPay expects amounts in VND cents × 100. |
| QR code expired | Intents expire after 15 minutes; regenerate via the checkout button. |

For additional scenarios, consult the official VNPay API documentation and keep this file updated whenever the integration changes.

