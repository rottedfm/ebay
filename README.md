# ✅ Rust eBay Scraper - Simple TODO

## 🎯 Goal
Log into eBay (with CAPTCHA solving), scrape your listings, and save them to a SQL database.

---

## 📦 Setup
- [X] Create Rust project: `cargo init ebay_scraper`
- [X] Add crates: `fantoccini`, `tokio`, `reqwest`, `serde`, `sqlx` or `rusqlite`, `dotenv`, `clap`, `log`

---

## 🧪 CLI
- [X] Add CLI commands:
  - `login` – Login with CAPTCHA solving
  - `scrape` – Scrape listings
  - `sync` – Scrape + save to DB
  - `all` – Run full flow

---

## 🔐 Login Flow
- [ ] Use `fantoccini` to open eBay login page
- [ ] Solve CAPTCHA using 2Captcha API
- [ ] Enter credentials and login
- [ ] (Optional) Save session/cookies

---

## 📊 Scrape Listings
- [ ] Navigate to listings page
- [ ] Extract title, price, qty, condition, ID, URL
- [ ] Store data in `Listing` struct

---

## 🗄️ Save to DB
- [ ] Initialize SQLite DB
- [ ] Create `listings` table
- [ ] Insert or update each listing

---

## 🧰 Testing & Final Steps
- [ ] Test CAPTCHA handling
- [ ] Test login and scrape manually
- [ ] Check DB contents
- [ ] Add README + `.env.example`

