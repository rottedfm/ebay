use crate::client::Listing;
use anyhow::Result;
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::Path;

// A simplified version of `Listing` used for CSV serialization
#[derive(Debug, Serialize, Deserialize, Clone)]
struct CsvListing {
    pub title: String,
    pub item_id: String,
    pub price: String,
    pub views: String,
    pub watchers: String,
}

// Converts a `Listing` to a `CsvListing` for CSV-friendly output
impl From<&Listing> for CsvListing {
    fn from(listing: &Listing) -> Self {
        CsvListing {
            title: listing.title.clone(),
            item_id: listing.item_id.clone(),
            price: listing.price.clone(),
            views: listing.views.clone(),
            watchers: listing.watchers.clone(),
        }
    }
}

// Write a list of new listings to a CSV file, preserving any previously existing data
pub fn write_listings_to_csv(new_listings: &[Listing], csv_path: &str) -> Result<()> {
    // A map of all listings keyed by item_id, used to deduplicate and overwrite old entries
    let mut map: HashMap<String, CsvListing> = HashMap::new();

    // Load existing records if file exists
    if Path::new(csv_path).exists() {
        let file = File::open(csv_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
        for result in rdr.deserialize::<CsvListing>() {
            let entry = result?;
            map.insert(entry.item_id.clone(), entry);
        }
    }

    // Overwrite or insert new entries
    for listing in new_listings {
        let entry = CsvListing::from(listing);
        map.insert(entry.item_id.clone(), entry);
    }

    // Write synced listings to file, once
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(csv_path)?;

    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);

    // Write all the current values in the map back to the file
    for entry in map.values() {
        wtr.serialize(entry)?;
    }

    // Ensure all data is flushed to disk
    wtr.flush()?;
    Ok(())
}
