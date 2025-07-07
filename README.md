# eSd - eBay Seller Dashboard

## 📜 Terms of Use & Compliance

To ensure this application remains fully compliant with eBay's terms, policies, and API license agreement, the following guidelines and conditions must be followed at all times:

### ✅ Scope of Use
This tool is intended for use by a single, authenticated eBay seller account. It manages your own listings, feedback, offers, and inventory through the eBay API or (where necessary) limited, respectful scraping of your own seller dashboard.

### 📦 Listing Compliance
Listings must follow eBay’s global listing policies:
- Only legal, permitted, and accurately described items may be listed.
- Listings must not include prohibited content such as counterfeit goods, personal contact info, or off-site links.
- Prices and shipping fees must be transparent and not misleading.
- Stock levels must be accurate; end listings promptly when items sell out.

### 🧾 Listing Structure & Content
- Titles and descriptions must truthfully reflect the item.
- Images must be of the actual item unless explicitly disclosed as stock photos.
- No duplicate listings unless allowed via variation formats.
- Auction and Buy It Now formats must not be used to manipulate fees or rankings.

### 🔄 Inventory & Offer Handling
This tool may:
- List, revise, and end items using the eBay Inventory or Trading API.
- Handle offers via the eBay Negotiation API.
- Manage pricing, stock levels, and return policies.

Offer and pricing updates must follow eBay's dynamic pricing and fair use standards. Shill bidding and coordinated activity are strictly prohibited.

### ⭐ Feedback Management
Feedback is displayed using the authenticated seller’s data. When stored, it must be refreshed every 6 hours. Public display of other users’ feedback or scraping buyer reviews is not allowed.

### 🔐 API Access & Token Use
You must:
- Use secure OAuth authentication and store access tokens safely.
- Never expose App ID or secrets in logs, UIs, or shared code.
- Comply with all rate limits and refresh tokens before expiration.
- Only use API endpoints for your own data and account.

API usage must respect eBay’s Developer Program Agreement, including restrictions on aggregation, sharing data externally, or analyzing competitive data at scale.

### 🧠 Data Storage & Privacy
- All scraped or API-fetched data must be refreshed regularly.
- Do not retain data beyond permitted intervals (e.g., 24 hrs for listings, 6 hrs for feedback).
- Personal or sensitive data must be encrypted and access-controlled.
- If scraping is used, it must only target your account’s public dashboard or listings.

No buyer-identifiable information may be stored, displayed, or exported.

### 🧰 TUI App Behavior
The terminal-based application:
- Must act only on behalf of the authenticated user.
- May automate inventory syncing, feedback responses, or offer management.
- Should log listing activity and maintain auditability.
- Must not auto-post content or operate without user control or consent.

### ⚠️ Prohibited Activities
- Scraping other sellers' data.
- Displaying, sharing, or storing data from unauthorized accounts.
- Circumventing eBay rate limits or anti-bot systems.
- Using the application to manipulate search, rankings, or pricing.

### ✅ Summary
Use the official eBay API wherever possible. Scraping should only be used as a fallback for your own account data. Respect all privacy, rate limits, and security rules. By using this tool, you agree to comply with eBay’s [API License Agreement](https://developer.ebay.com/join/api-license-agreement), [Seller Policies](https://www.ebay.com/help/policies), and applicable local laws.

