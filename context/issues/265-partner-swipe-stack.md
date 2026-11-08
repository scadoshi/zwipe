# 265. Partner Swipe stack still offers illegal partners

State: reproduced | Owner: laptop | PR: - | Build seen: 1e03520ef

## Report
Approved nit 265 (#399): the partner list offers only legal partners. Verification failed in part.

## Repro
Surfer, live 1e03520ef: with Tymna the Weaver as commander, the Partner typeahead is right (Thrasios and The Prismatic Piper offered; "Pir" and "Ian Ches" give nothing). The Partner **Swipe** stack still offers Toothy, Slurrk, Ukkima, Brallin, Shabraz and Sam (Partner with) and Wernog and Cecily (Friends forever) in its first 22 cards.
