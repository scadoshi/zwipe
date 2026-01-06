1. Explanation: NO. It should be computed at time of querying the database on the database layer when the user passes in the "is_playable: Some(false)" in the CardFilter. So the ddatabase just adds into *its* query to ignore layouts of these types. Don't store oconstant in domain layer. This can live in the database layer. Either as a constant if you think that is best OR directly written into the query that is fine.

2. Answer: A. Explanation: Let's just serve english cards for now. We should have a CardFilter attribute called "language: Language" which should be an enumeration of the supported languages with their correlated language codes translated at the database level only. So today Languages will look something like:

```rust 
#[derive(Debug)]
public enum Language {
    English
}
```

And as I add language support I will add more languages to this. The database will have the responsibility of translating `Language:English` to `'en'`. 

3. Answer: A. Explanation: Users can use Arena to track their Arena decks so let's NOT support digital or alchemy for now please.

4. Answer: A. Explanation: Doesn't this render a card "unplayable"? Should this not just be added to the unplayable filter? 

5. Answer: A. Explanation: We should probably build a legality filter that shows all formats playable in physical. This is going to be more of an effort but we should add it. Won't be too hard. More formats aren't going to really be added so we can build an enum for it. Wait oh my god I see that I already have an enum for it haha. Okay so we should definitely filter on legality. But it is a bit more complicated than first thought as Legality doesn't only have the actual format but also the LegalityKind including options like Banned, Legal, etc. so we will have to build a special type of filter in the card filter front end. Let's focus this change on the card filter object and refine this change plan. Before we commit to what we're going to change, because this is exploding a bit (which is fine) we should outline all of the changes today in @.context/project/next.md outlining the overall goal of serving only playable cards with CardFilter and add other useful filters liek legality/format. Basically add the fully scoped tasks to next.md and then decrease scope of what we're doing right now so we do less in each commit. Maybe we just show set_type as a filter and allow the user to filter on unsets or funny sets if they want to but DEFAULT the card_filter to do Some('funny') for set type. But let's just plan this all out for now.

6. Answer: A. Explanation: I think i might just allow the user to select filter of all of the set types as they ALL have some cards with images. But will default that filter to HIDE certain set types. Let's add to CardFilter, default CardFilter to NOT show these set_types, and for NOW NOT SHOW this set_type as a toggle-able filter on the frontend. We will want to make a log of all newly added CardFilter things that we might want to expose on the frontend later if we wanted though.

7. Answer: A. Explanation: Add to CardFilter. Default to Some(false). Don't show on frontend for now.

8. Same answer as 7. 

9. What do you mean does the backend return them? The backend will always serve the entire card object with all of the values we're talking about. CardFilter will include a lot of these for future use or frontend internal use but often not be exposed to the end user for now. 

10. You can check migrations in zwipe/zerver/src/migrations.. for this. I have checked all fields in the database myself with queries and they are in there. 
