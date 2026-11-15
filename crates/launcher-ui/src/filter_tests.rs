use crate::filter::{filter_sort, Sort};
use crate::model::{Card, CardState};

fn card(id: &str, name: &str, description: &str) -> Card {
    Card {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        installed: None,
        latest: Some("1.0.0".into()),
        state: CardState::NotInstalled,
        date: None,
        license: String::new(),
        author: String::new(),
        running: false,
    }
}

fn cards() -> Vec<Card> {
    vec![
        card("z", "Zeichenbrett", "Skizzen und Ideen."),
        card("a", "Ärztebuch", "Termine im Blick."),
        card("n", "Notizbuch", "Kurze Notizen sammeln."),
    ]
}

fn names(list: &[&Card]) -> Vec<String> {
    list.iter().map(|c| c.name.clone()).collect()
}

#[test]
fn ohne_suche_sind_alle_apps_sichtbar_und_nach_name_sortiert() {
    let all = cards();
    let list = filter_sort(&all, "", Sort::NameAsc);
    assert_eq!(names(&list), ["Ärztebuch", "Notizbuch", "Zeichenbrett"]);
    let list = filter_sort(&all, "   ", Sort::NameDesc);
    assert_eq!(names(&list), ["Zeichenbrett", "Notizbuch", "Ärztebuch"]);
}

#[test]
fn suche_findet_in_name_und_kurztext_ohne_gross_und_kleinschreibung() {
    let all = cards();
    assert_eq!(
        names(&filter_sort(&all, "NOTIZ", Sort::NameAsc)),
        ["Notizbuch"]
    );
    assert_eq!(
        names(&filter_sort(&all, "ideen", Sort::NameAsc)),
        ["Zeichenbrett"]
    );
    assert_eq!(
        names(&filter_sort(&all, " buch ", Sort::NameAsc)),
        ["Ärztebuch", "Notizbuch"]
    );
}

#[test]
fn ohne_treffer_ist_die_liste_leer() {
    assert!(filter_sort(&cards(), "xyz", Sort::NameAsc).is_empty());
}
