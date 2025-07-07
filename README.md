# eSd - eBay Seller Dashboard

## IMPORTANT!!!
- [eBay API Terms Of Service](https://developer.ebay.com/join/api-license-agreement)
- [eBay Terms Of Service](https://www.ebay.com/help/policies/member-behaviour-policies/user-agreement?id=4259)

### 1. Allowed Items Only (Listing Policy)
- [ ] Item is **not prohibited** (e.g., weapons, counterfeit goods, recalled items, etc.)
- [ ] You have the **legal right to sell** the item (e.g., licenses, trademarks, ownership)
- [ ] You are not violating any **brand restrictions or VERO violations**


### 2. Accurate Listings
- [ ] Title reflects the item (no clickbait or keyword spamming)
- [ ] Photos are accurate and match the product (or disclose if stock photo)
- [ ] Condition is correctly set ("New", "Used", etc.)
- [ ] Item specifics (brand, size, etc.) are honest and complete


### 3. Pricing & Terms
- [ ] Pricing is transparent and fair (no $0.01 bait-and-switch shipping tricks)
- [ ] Shipping costs and timelines are accurately described
- [ ] Return policies are clearly stated and honored
- [ ] Customs/tax info is disclosed for international buyers

### 4. Content Restrictions
- [ ] No off-site links or personal contact info in the listing
- [ ] No attempts to take the transaction **off-eBay**
- [ ] No duplicate or miscategorized listings


### 5. Inventory & Fulfillment
- [ ] Only list items you **actually have in stock**
- [ ] End listings promptly if out of stock
- [ ] Upload tracking info on time (if applicable)
- [ ] Accurately reflect handling time and shipping service

### 6. Listing Format Rules
- [ ] Auction starting prices are fair
- [ ] Buy It Now pricing is not misleading
- [ ] No shill bidding or listing manipulation
- [ ] Do not use bots or automation to game visibility or promote items unfairly


### 7. Feedback Handling (via UI or API)
- [ ] Feedback shown is refreshed at least every **6 hours**
- [ ] You do not store or display other users' feedback data
- [ ] You respond to feedback professionally and according to eBay rules


### 8. Offer Management
- [ ] Offers handled via the [Negotiation API](https://developer.ebay.com/api-docs/sell/negotiation/resources/offer) or within your seller dashboard
- [ ] Buyer usernames or info is handled securely
- [ ] Offers are refreshed regularly and not stored long-term


### 9. eBay API Usage (Developer Compliance)

#### Authentication
- [ ] You use your own eBay **App ID, Cert ID, and OAuth token**
- [ ] Tokens are stored securely (e.g., `.env` or encrypted config)
- [ ] You use OAuth 2.0 for any sensitive data access

#### Rate Limits & App Behavior
- [ ] You respect all **rate limits** (per-app and per-endpoint)
- [ ] You do not share your API keys or tokens with other users
- [ ] You do not bulk scrape or perform data mining beyond your account

#### Data Freshness Rules
| Data Type     | Refreshed Every |
|---------------|------------------|
| Listings      | [ ] ≤ 24h ✔️     |
| Feedback      | [ ] ≤ 6h ✔️      |
| Offers        | [ ] ≤ 24h ✔️     |

####  Data Storage
- [ ] Only necessary data is stored
- [ ] Personal or buyer data is **not stored long-term**
- [ ] You comply with the [DPRA Addendum](https://developer.ebay.com/api-docs/static/rest-request-components.html#data-protection-and-security) if storing personal data
- [ ] You delete expired or invalidated tokens/data promptly


### 10. Automation & TUI App Behavior
- [ ] Your TUI app **only manages your own account**
- [ ] It does not expose, sync, or display other sellers' listings
- [ ] You have a mechanism to **refresh or resync** listing/feedback data
- [ ] You log changes (create/update/end listings) for auditability
- [ ] You avoid calling APIs for data already cached unless expired


### 11. Security & Ethics
- [ ] API credentials and tokens are never hardcoded or publicly visible
- [ ] Buyer/user data is not logged, shared, or misused
- [ ] You publish a privacy policy if user tokens or personal data are stored
- [ ] You delete sensitive data within 10 days of token expiration or app deactivation


### Bonus Best Practices
- [ ] Use a local TTL (time-to-live) cache for listings/feedback
- [ ] Display last sync timestamp in your TUI
- [ ] Offer manual "Resync" option for stale data
- [ ] Use item IDs as the primary key in your DB or display list
- [ ] Respect listing insertion fees and do not auto-list in bulk without user review





## DELETE - What do we need
POSTED
- Offer bot (watched = decrease listing price, Then offer set discount)
- Feedback bot
- Check every listing for discolusers

UNPOSTED
- read pictures
- research item with chatgpt
- make checklist for photo quality
- ask user for codition
- generate different keywords for to look for ebay listings of the same item
- generate profit maximized price


