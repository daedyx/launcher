use crate::model::Card;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    #[default]
    NameAsc,
    NameDesc,
}

/// Wendet Suche (Name und Kurztext, ohne Groß-/Kleinschreibung) und Sortierung an.
pub fn filter_sort<'a>(cards: &'a [Card], query: &str, sort: Sort) -> Vec<&'a Card> {
    let q = query.trim().to_lowercase();
    let mut list: Vec<&Card> = cards
        .iter()
        .filter(|c| {
            q.is_empty()
                || c.name.to_lowercase().contains(&q)
                || c.description.to_lowercase().contains(&q)
        })
        .collect();
    list.sort_by_key(|c| sort_key(&c.name));
    if sort == Sort::NameDesc {
        list.reverse();
    }
    list
}

/// Sortierschlüssel nach deutscher Gewohnheit: „Ärztebuch“ steht bei A, „Straße“ bei „Strasse“.
fn sort_key(name: &str) -> String {
    name.to_lowercase()
        .replace('ä', "a")
        .replace('ö', "o")
        .replace('ü', "u")
        .replace('ß', "ss")
}
