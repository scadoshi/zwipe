use crate::models::card::{card_profile::CardProfile, scryfall_card::ScryfallCard};
use sqlx::{query, query_as, PgPool};

pub trait Insert {
    async fn insert(
        &self,
        pg_pool: &PgPool,
        // State(app_state): State<AppState>,
    ) -> Result<CardProfile, sqlx::Error>;
}

impl Insert for ScryfallCard {
    async fn insert(
        &self,
        pg_pool: &PgPool,
        // State(app_state): State<AppState>,
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
                $75, $76, $77, $78, $79
            ) RETURNING *
            "#,
            // Core Card Fields
            // Cards have the following core properties
            self.arena_id,
            self.id,
            self.lang,
            self.mtgo_id,
            self.mtgo_foil_id,
            self.multiverse_ids.as_deref(),
            self.tcgplayer_id,
            self.tcgplayer_etched_id,
            self.cardmarket_id,
            self.object,
            self.layout,
            self.oracle_id,
            self.prints_search_uri,
            self.rulings_uri,
            self.scryfall_uri,
            self.uri,
    
            // Gameplay Fields
            // Cards have the following properties relevant to the game rules
            // scryfall_card.all_parts
            // scryfall_card.card_faces
            self.cmc,
            &self.color_identity,
            self.color_indicator.as_deref(),
            self.colors.as_deref(),
            self.defense,
            self.edhrec_rank,
            self.game_changer,
            self.hand_modifier,
            self.keywords.as_deref(),
            // scryfall_card.legalities
            self.life_modifier,
            self.loyalty,
            self.mana_cost,
            self.name,
            self.oracle_text,
            self.penny_rank,
            self.power,
            self.produced_mana.as_deref(),
            self.reserved,
            self.toughness,
            self.type_line,
    
            // Print Fields
            // Cards have the following properties unique to their particular re/print
            self.artist,
            self.artist_ids.as_deref(),
            self.booster,
            self.border_color,
            self.card_back_id,
            self.collector_number,
            self.content_warning,
            self.digital,
            &self.finishes,
            self.flavor_name,
            self.flavor_text,
            self.frame_effects.as_deref(),
            self.frame,
            self.full_art,
            self.games.as_deref(),
            self.highres_image,
            self.illustration_id,
            self.image_status,
            // scryfall_card.image_uris
            self.oversized,
            // scryfall_card.prices: Prices,
            self.printed_name,
            self.printed_text,
            self.printed_type_line,
            self.promo,
            self.promo_types.as_deref(),
            // scryfall_card.purchase_uris // fix later
            self.rarity,
            // scryfall_card.related_uris // fix later
            self.released_at,
            self.reprint,
            self.scryfall_set_uri,
            self.set_name,
            self.set_search_uri,
            self.set_type,
            self.set_uri,
            self.set,
            self.set_id,
            self.story_spotlight,
            self.textless,
            self.variation,
            self.variation_of,
            self.security_stamp,
            self.watermark,
            self.preview_previewed_at,
            self.preview_source_uri,
            self.preview_source,
        )
        .fetch_one(pg_pool)
        .await?;
    
        // println!("(*3*)<(added {:?} to the database!)", sf_card.name);
    
        let card_profile = query_as!(
            CardProfile,
            r"INSERT INTO card_profiles (scryfall_card_id, created_at, updated_at) VALUES ($1, $2, $3) RETURNING *",
            self.id,
            chrono::Utc::now().naive_utc(),
            chrono::Utc::now().naive_utc(),
        )
        .fetch_one(pg_pool)
        .await?;
    
        // println!(
        //     "(*3*)<(the profile for {:?} can be found at id = {:?})",
        //     sf_card.name, card_profile.id
        // );
    
        Ok(card_profile)
    }
    
}
pub async fn delete_all_cards(pg_pool: &PgPool) -> Result<(), sqlx::Error> {
    query!("DELETE FROM scryfall_cards;")
        .execute(pg_pool)
        .await?;
    Ok(())
}
