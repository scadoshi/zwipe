// @generated automatically by Diesel CLI.

diesel::table! {
    cards (id) {
        id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        arena_id -> Nullable<Int4>,
        scryfall_id -> Uuid,
        lang -> Varchar,
        mtgo_id -> Nullable<Int4>,
        mtgo_foil_id -> Nullable<Int4>,
        multiverse_ids -> Nullable<Array<Nullable<Int4>>>,
        tcgplayer_id -> Nullable<Int4>,
        tcgplayer_etched_id -> Nullable<Int4>,
        cardmarket_id -> Nullable<Int4>,
        object -> Varchar,
        layout -> Varchar,
        oracle_id -> Nullable<Uuid>,
        prints_search_uri -> Varchar,
        rulings_uri -> Varchar,
        scryfall_uri -> Varchar,
        scryfall_api_uri -> Varchar,
        cmc -> Float8,
        color_identity -> Nullable<Array<Nullable<Text>>>,
        color_indicator -> Nullable<Array<Nullable<Text>>>,
        colors -> Nullable<Array<Nullable<Text>>>,
        defense -> Nullable<Varchar>,
        edhrec_rank -> Nullable<Int4>,
        game_changer -> Nullable<Bool>,
        hand_modifier -> Nullable<Varchar>,
        keywords -> Nullable<Array<Nullable<Text>>>,
        life_modifier -> Nullable<Varchar>,
        loyalty -> Nullable<Varchar>,
        mana_cost -> Nullable<Varchar>,
        name -> Varchar,
        oracle_text -> Nullable<Varchar>,
        penny_rank -> Nullable<Int4>,
        power -> Nullable<Varchar>,
        produced_mana -> Nullable<Jsonb>,
        reserved -> Bool,
        toughness -> Nullable<Varchar>,
        type_line -> Varchar,
        artist -> Nullable<Varchar>,
        artist_ids -> Nullable<Array<Nullable<Uuid>>>,
        attraction_lights -> Nullable<Array<Nullable<Text>>>,
        booster -> Bool,
        border_color -> Varchar,
        card_back_id -> Uuid,
        collector_number -> Varchar,
        content_warning -> Nullable<Bool>,
        digital -> Bool,
        finishes -> Array<Nullable<Text>>,
        flavor_name -> Nullable<Varchar>,
        flavor_text -> Nullable<Varchar>,
        frame_effects -> Nullable<Array<Nullable<Text>>>,
        frame -> Varchar,
        full_art -> Bool,
        games -> Nullable<Array<Nullable<Text>>>,
        highres_image -> Bool,
        illustration_id -> Nullable<Uuid>,
        image_status -> Varchar,
        oversized -> Bool,
        printed_name -> Nullable<Varchar>,
        printed_text -> Nullable<Varchar>,
        printed_type_line -> Nullable<Varchar>,
        promo -> Bool,
        promo_types -> Nullable<Array<Nullable<Text>>>,
        rarity -> Varchar,
        released_at -> Date,
        reprint -> Bool,
        scryfall_set_uri -> Varchar,
        set_name -> Varchar,
        set_search_uri -> Varchar,
        set_type -> Varchar,
        set_uri -> Varchar,
        set -> Varchar,
        set_id -> Uuid,
        story_spotlight -> Bool,
        textless -> Bool,
        variation -> Bool,
        variation_of -> Nullable<Uuid>,
        security_stamp -> Nullable<Varchar>,
        watermark -> Nullable<Varchar>,
        preview_previewed_at -> Nullable<Date>,
        preview_source_uri -> Nullable<Varchar>,
        preview_source -> Nullable<Varchar>,
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
