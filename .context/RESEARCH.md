Conversation Summary: Non-Playable Card Layout Filtering Strategy
Question: Should card filtering use whitelist (include playable layouts) or blacklist (exclude non-playable layouts) approach for is_playable filter?
Complete Non-Playable Layouts Identified: token, double_faced_token, emblem, art_series, vanguard, scheme, planar, augment, host.
Complete Playable Layouts Identified: normal, split, flip, transform, modal_dfc, meld, reversible_card, leveler, saga, adventure, mutate, prototype, battle, class, case. Note: double_faced_token debatable—some Incubator tokens transform into creatures but aren't deck-building cards.
Analysis: Wizards frequently introduces new non-playable gimmick formats (Vanguard 1997, Planes/Schemes 2009-2012, Unstable augment/host 2017, art_series 2020s). Playable card mechanics are stable and rare—new playable layouts (modal_dfc, adventure, battle, class, case, prototype) occur infrequently with major set releases and significant announcement.
Whitelist Advantages: (1) Unknown layouts default to hidden—safe behavior prevents garbage data in production, (2) New playable mechanics require intentional review and approval before user exposure, (3) Reduces maintenance burden—only update when Wizards announces major new mechanic, (4) Prevents support tickets from unexpected non-playable content appearing.
Blacklist Disadvantages: (1) New gimmick formats appear unannounced in supplemental products, (2) Constant reactive maintenance chasing Wizards experiments, (3) Unknown layouts default to visible—risky for data quality, (4) One missed layout causes user confusion and data cleanup.
Recommended Implementation: Use PLAYABLE_LAYOUTS whitelist set containing 15 known playable layouts. Filter returns False for any layout not in whitelist. Include additional safety check for malformed entries where name contains "//" but card_faces missing or length <=1. For deck-building focused apps exclude double_faced_token from whitelist. Maintenance workflow: new unknown layout auto-excluded, investigate on data ingestion, add to whitelist if playable after verification, ignore if non-playable.
Conclusion: Whitelist approach strongly recommended. Playable set well-defined and changes infrequently with major announcements. Non-playable set is Wizards creative playground with unpredictable additions. Default-safe behavior critical for production data quality.
Conversation Summary: Additional Card Filtering Dimensions Beyond is_playable
Context: is_playable layout filter is foundational but insufficient alone. Applications require multiple filtering dimensions based on use case.
Additional Filtering Dimensions Identified:
1. Legality Filters: Format legality (Standard, Modern, Pioneer, Commander, Legacy, Vintage, Pauper), banned/restricted status. Use legalities field on card objects.
2. Language Filters: English vs other languages. Use lang field. Most apps default to lang=="en" for primary display to avoid duplicate functional cards in different languages.
3. Set Type Filters: Core sets vs expansions vs promos vs tokens vs memorabilia. Use set_type field on set object. Common filters exclude memorabilia, funny sets, token sets.
4. Digital-Only Filters: Paper vs Arena vs MTGO exclusive cards. Use digital boolean and games array. Arena-only cards include Alchemy rebalanced versions not available in paper.
5. Reprint/Uniqueness Filters: First printing vs reprints. Use reprint boolean. Oracle uniqueness (unique card name/rules) vs printing uniqueness (specific edition/art variant).
6. Promo/Special Printings: Promo cards, alternate art, extended art, borderless variants. Use promo, border_color, frame_effects, variation fields. Users may want cheapest printing or newest printing filtering.
7. Oversized/Physical Format: Oversized cards (commanders, planechase planes) that can't fit in sleeves/decks. Use oversized boolean. Physically unplayable despite having game rules.
8. Funny/Un-set Cards: Silver-bordered, acorn stamp cards from Un-sets and joke products. Check set_type for funny or equivalent. Not tournament legal, parody/joke cards.
9. Reserved List: Cards on Reserved List affecting reprinting/availability/price. Use reserved boolean. Relevant for collection value and acquisition planning.
10. Content Warnings: Cards with disturbing imagery flagged by Scryfall. Use content_warning boolean. Some apps hide by default for user experience.
Common Filter Combination Patterns:
Standard Playable Filter: is_playable AND lang=="en" AND not digital AND not oversized AND legalities.standard in [legal,restricted] AND set_type not funny. Most restrictive tournament-ready cards only.
Deck Building Filter: is_playable AND lang=="en" AND not oversized AND set_type not in [memorabilia,token,funny]. For deck builder apps showing real cards users can add to decks.
Collection Tracking Filter: is_playable AND not digital AND not oversized AND "paper" in games. For collection tracking including promos and alternate arts but only physical obtainable cards.
Key Insight: Different application use cases (tournament preparation, casual deck building, collection management, price tracking, card database) require different filter combinations. is_playable is necessary foundation but insufficient alone. Applications should compose multiple filter dimensions based on specific user needs and context.

1. Yes this format is fine. It SHOULD defualt to Some(false)
  however. 2. No. We can determine it by saying layout = 'art_series'
  or layout = != 'art_series' 3. Latest research on this in
  @.context/RESEARCH.md there are other non-playable cards that we
  should ignore. I was thinking about building an enum to handle this
  but I don't want syncs to break when they add new ones. So how about
  building a constant on the domain layer that contains a comma
  separated list of playable and non-playable layout texts that are
  encodeable in the database layer to see if the card matches those
  layouts when serving. I found even more factors about a card that
  would indicate that I should not serve them to the user. Please
  evaluate these in depth and ask me questions about each one
  determing whether or not that I WANT TO HIDE THEM OR NOT TO KEEP
  YOUR EVALUATION DETERMINISTIC OF WHAT I WANT AND HOW WE WANT TO
  BUILD THIS. 4. skip this for now.
  I found the only two layo 4.  
I found the only two layouts that have matching faces: zerver=> 
zerver=> select * from (SELECT DISTINCT
    layout,
    (
        TRIM(BOTH '/ ' FROM SPLIT_PART(name, '//', 1))
        =
        TRIM(BOTH '/ ' FROM SPLIT_PART(name, '//', 2))
    ) AS names_match
FROM scryfall_data) a where names_match = true;
       layout       | names_match 
--------------------+-------------
 art_series         | t
 double_faced_token | t

it is JUST art_series and double_faced_token both are non-playable so we can ignore this edge case completely in question  4. 

5. Pivot we will build an is_playable filter in card_filter which is defaulted ON when the user gets cards for get cards for adding to their deck but remember we will also now have other filters explained in the above
