// @generated automatically by Diesel CLI.

diesel::table! {
    cards (id) {
        id -> Int4,
        name -> Varchar,
        mana_cost -> Nullable<Varchar>,
        card_type -> Varchar,
        rarity -> Varchar,
        image_url -> Varchar,
        oracle_text -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    deck_cards (id) {
        id -> Int4,
        deck_id -> Int4,
        card_id -> Int4,
        quantity -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    decks (id) {
        id -> Int4,
        name -> Varchar,
        format -> Varchar,
        user_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        username -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(deck_cards -> cards (card_id));
diesel::joinable!(deck_cards -> decks (deck_id));
diesel::joinable!(decks -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    cards,
    deck_cards,
    decks,
    users,
);
