use crate::models::card::{card_profile::CardProfile, scryfall_card::ScryfallCard};
use sqlx::{query_as, PgPool};

pub async fn insert_card(
    // State(app_state): State<AppState>,
    pg_pool: &PgPool,
    scryfall_card: ScryfallCard,
) -> Result<CardProfile, sqlx::Error> {
    // add these fields later
    // will have to build structs for them
    // make sure their order matches what is in mod.rs as you add
    // all_parts,
    // card_faces,
    // legalities
    // image_uris,
    // prices: Prices,
    // purchase_uris,
    // related_uris,
    let sf_card = query_as!(
        ScryfallCard,
        r#"
        INSERT INTO scryfall_cards (
            arena_id,
            id,
            lang,
            mtgo_id,
            mtgo_foil_id,
            multiverse_ids,
            tcgplayer_id,
            tcgplayer_etched_id,
            cardmarket_id,
            object,
            layout,
            oracle_id,
            prints_search_uri,
            rulings_uri,
            scryfall_uri,
            uri,
            cmc,
            color_identity,
            color_indicator,
            colors,
            defense,
            edhrec_rank,
            game_changer,
            hand_modifier,
            keywords,
            life_modifier,
            loyalty,
            mana_cost,
            name,
            oracle_text,
            penny_rank,
            power,
            produced_mana,
            reserved,
            toughness,
            type_line,
            artist,
            artist_ids,
            attraction_lights,
            booster,
            border_color,
            card_back_id,
            collector_number,
            content_warning,
            digital,
            finishes,
            flavor_name,
            flavor_text,
            frame_effects,
            frame,
            full_art,
            games,
            highres_image,
            illustration_id,
            image_status,
            oversized,
            printed_name,
            printed_text,
            printed_type_line,
            promo,
            promo_types,
            rarity,
            released_at,
            reprint,
            scryfall_set_uri,
            set_name,
            set_search_uri,
            set_type,
            set_uri,
            set,
            set_id,
            story_spotlight,
            textless,
            variation,
            variation_of,
            security_stamp,
            watermark,
            preview_previewed_at,
            preview_source_uri,
            preview_source
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37, $38,
                $39, $40, $41, $42, $43, $44, $45, $46, $47, $48, $49, $50, $51, $52, $53, $54, $55, $56,
            $57, $58, $59, $60, $61, $62, $63, $64, $65, $66, $67, $68, $69, $70, $71, $72, $73, $74,
            $75, $76, $77, $78, $79, $80
        ) RETURNING *
        "#,
        // Core Card Fields
        // Cards have the following core properties
        scryfall_card.arena_id,
        scryfall_card.id,
        scryfall_card.lang,
        scryfall_card.mtgo_id,
        scryfall_card.mtgo_foil_id,
        scryfall_card.multiverse_ids.as_deref(),
        scryfall_card.tcgplayer_id,
        scryfall_card.tcgplayer_etched_id,
        scryfall_card.cardmarket_id,
        scryfall_card.object,
        scryfall_card.layout,
        scryfall_card.oracle_id,
        scryfall_card.prints_search_uri,
        scryfall_card.rulings_uri,
        scryfall_card.scryfall_uri,
        scryfall_card.uri,

        // Gameplay Fields
        // Cards have the following properties relevant to the game rules
        // scryfall_card.all_parts
        // scryfall_card.card_faces
        scryfall_card.cmc,
        &scryfall_card.color_identity,
        scryfall_card.color_indicator.as_deref(),
        scryfall_card.colors.as_deref(),
        scryfall_card.defense,
        scryfall_card.edhrec_rank,
        scryfall_card.game_changer,
        scryfall_card.hand_modifier,
        scryfall_card.keywords.as_deref(),
        // scryfall_card.legalities
        scryfall_card.life_modifier,
        scryfall_card.loyalty,
        scryfall_card.mana_cost,
        scryfall_card.name,
        scryfall_card.oracle_text,
        scryfall_card.penny_rank,
        scryfall_card.power,
        scryfall_card.produced_mana.as_deref(),
        scryfall_card.reserved,
        scryfall_card.toughness,
        scryfall_card.type_line,

        // Print Fields
        // Cards have the following properties unique to their particular re/print
        scryfall_card.artist,
        scryfall_card.artist_ids.as_deref(),
        scryfall_card.attraction_lights.as_deref(),
        scryfall_card.booster,
        scryfall_card.border_color,
        scryfall_card.card_back_id,
        scryfall_card.collector_number,
        scryfall_card.content_warning,
        scryfall_card.digital,
        &scryfall_card.finishes,
        scryfall_card.flavor_name,
        scryfall_card.flavor_text,
        scryfall_card.frame_effects.as_deref(),
        scryfall_card.frame,
        scryfall_card.full_art,
        scryfall_card.games.as_deref(),
        scryfall_card.highres_image,
        scryfall_card.illustration_id,
        scryfall_card.image_status,
        // scryfall_card.image_uris
        scryfall_card.oversized,
        // scryfall_card.prices: Prices,
        scryfall_card.printed_name,
        scryfall_card.printed_text,
        scryfall_card.printed_type_line,
        scryfall_card.promo,
        scryfall_card.promo_types.as_deref(),
        // scryfall_card.purchase_uris // fix later
        scryfall_card.rarity,
        // scryfall_card.related_uris // fix later
        scryfall_card.released_at,
        scryfall_card.reprint,
        scryfall_card.scryfall_set_uri,
        scryfall_card.set_name,
        scryfall_card.set_search_uri,
        scryfall_card.set_type,
        scryfall_card.set_uri,
        scryfall_card.set,
        scryfall_card.set_id,
        scryfall_card.story_spotlight,
        scryfall_card.textless,
        scryfall_card.variation,
        scryfall_card.variation_of,
        scryfall_card.security_stamp,
        scryfall_card.watermark,
        scryfall_card.preview_previewed_at,
        scryfall_card.preview_source_uri,
        scryfall_card.preview_source,
    )
    .fetch_one(pg_pool)
    .await?;

    println!("(*3*)<(added {:?} to the database!)", sf_card.name);

    let card_profile = query_as!(
        CardProfile,
        r"INSERT INTO card_profiles (scryfall_card_id, created_at, updated_at) VALUES ($1, $2, $3) RETURNING *",
        scryfall_card.id,
        chrono::Utc::now().naive_utc(),
        chrono::Utc::now().naive_utc(),
    )
    .fetch_one(pg_pool)
    .await?;

    println!(
        "(*3*)<(the profile for {:?} can be found at id = {:?})",
        sf_card.name, card_profile.id
    );

    Ok(card_profile)
}
