# Roadmap principale — révision post-V4
## Stretch : trajectoire corrigée à partir du système réellement obtenu

## 1. Objet

Ce document remplace la roadmap précédente à partir de la sortie réelle de la V4.

Il prend en compte :

- les aboutissements techniques effectifs de la V4 ;
- les instabilités encore ouvertes en V4 ;
- la vision post-V4 orientée vers la V5 ;
- la contrainte de versioning sans versions intermédiaires.

Le but n’est pas de prolonger mécaniquement la roadmap antérieure, mais de la recaler sur la réalité du système.

---

## 2. Ce qui est désormais acquis

### V0 — substrat dynamique minimal
- propagation locale ;
- dissipation ;
- traces ;
- plasticité locale ;
- corridors préférentiels.

### V1 — substrat spatial 3D non-grid
- graphe spatial 3D ;
- topologies KNN / radius ;
- KD-tree ;
- calibration 3D ;
- visualisation 3D ;
- formalisation mathématique.

### V2 — activité endogène régulée
- partitionnement spatial ;
- PID ;
- pacemakers ;
- consolidation structurelle ;
- activité auto-entretenue.

### V3 — dynamiques internes crédibles
- neurones E/I ;
- propagation signée ;
- PID indirect ;
- STDP ;
- budget synaptique compétitif ;
- scale 500k nœuds CPU ;
- substrat moins homogène.

### V4 — infrastructure d’apprentissage scalable
- architecture GPU-first ;
- pipeline de plasticité complet :
  STDP → éligibilité → 3 facteurs → homéostasie → consolidation → budget synaptique ;
- performance très supérieure à V3 ;
- compression mémoire forte ;
- exécution jusqu’à 5M nœuds et 57M arêtes ;
- portabilité GPU via wgpu.

---

## 3. Diagnostic post-V4

La V4 a résolu un bloc énorme d’infrastructure.
Stretch possède maintenant ses jambes : performance, échelle, pipeline, GPU compute.

Mais la V4 n’a pas encore prouvé que le système possède son cerveau.

### 3.1 Limites critiques observées

- le réseau fonctionne en régime flash & die ;
- l’activité n’est pas soutenue, mais pulsée ;
- l’accuracy 100% observée peut être expliquée par un biais topologique ;
- aucune preuve propre de chemins dopaminergiques n’a encore été démontrée ;
- les modifications de conductance restent faibles et symétriques ;
- l’éligibilité décroît trop vite par rapport au délai de reward ;
- le PID travaille sur un régime non stationnaire ;
- les paramètres ne scalent pas proprement de 50k à 5M ;
- l’infrastructure est validée, mais l’apprentissage véritable ne l’est pas encore.

---

## 4. Conséquence sur la roadmap

Le problème prioritaire n’est plus :
- la hiérarchie ;
- la spécialisation riche ;
- les assemblées ;
- la symbolisation.

Le problème prioritaire est désormais :

> prouver que le réseau apprend réellement au-delà du biais topologique.

Autrement dit, la V5 devient une version de validation scientifique du modèle, de calibration multi-échelle et de stabilisation dynamique.

---

## 5. Nouvelle logique de progression

```text
V4 acquis
→ V5 preuve d’apprentissage réelle + calibration multi-échelle + dynamique soutenue
→ V6 formation de chemins, mémoire guidée robuste et reward prediction minimale
→ V7 hiérarchie de zones et routage multi-échelle
→ V8 spécialisation neuronale riche et canaux multiples
→ V9 assemblées dynamiques et mémoire de travail
→ V10 chaînage, prédiction temporelle et proto-raisonnement
→ V11 symbolisation et interface discrète
→ V12 boucle linguistique externe minimale
→ V13 agent textuel émergent minimal
```

---

## 6. Roadmap consolidée

```text
V0   substrat dynamique minimal                               acquis
V1   espace 3D non-grid                                       acquis
V2   régulation locale et activité endogène                   acquis
V3   inhibition, STDP, PID indirect                           acquis
V4   dopamine, reward, éligibilité, I/O minimales + GPU-first acquis

V5   preuve d’apprentissage réelle, calibration multi-échelle, dynamique soutenue
V6   formation de chemins dopaminergiques, mémoire guidée robuste, reward prediction minimale
V7   hiérarchie de zones et routage multi-échelle
V8   spécialisation neuronale riche et canaux multiples
V9   assemblées dynamiques et mémoire de travail
V10  chaînage, prédiction temporelle et proto-raisonnement
V11  symbolisation et interface discrète
V12  boucle linguistique externe minimale
V13  agent textuel émergent minimal
```

---

# 7. V5 — Preuve d’apprentissage réelle, calibration multi-échelle, dynamique soutenue

## 7.1 Intention

La V5 doit répondre à une question unique et non négociable :

> le réseau apprend-il réellement, ou la topologie fait-elle encore le travail à sa place ?

La V5 devient donc la version de :
- preuve ;
- calibration ;
- stabilisation dynamique ;
- diagnostic scientifique.

## 7.2 Objectifs

- éliminer les biais topologiques des tâches ;
- prouver l’existence d’un apprentissage au-delà de la géométrie ;
- stabiliser une activité plus soutenue ;
- adapter les hyperparamètres à l’échelle ;
- fournir des outils diagnostics permettant de lire ce qui est appris.

## 7.3 Transformations majeures

### A. Tâches anti-biais topologique
- I/O symétriques ;
- associations inversées ;
- re-routing ;
- inversion de mapping après pré-entraînement.

### B. Baselines strictes
- random baseline ;
- topologie-only baseline ;
- full learning baseline.

### C. Calibration multi-échelle
Hyperparamètres adaptatifs selon `n`, `extent`, `group_size`, `read_delay`, etc.

### D. Dynamique soutenue
- decay adaptatif ;
- activité réverbérante locale ;
- connexions récurrentes ;
- présentations plus longues ;
- suppression ou réduction des resets destructeurs.

### E. Outils de compréhension
- heatmaps de conductance 3D ;
- path tracer ;
- timeline eligibility / conductance ;
- analyse de clusters de co-renforcement.

## 7.4 Ce que la V5 doit prouver

- l’accuracy reste > hasard sur des tâches sans biais topologique ;
- des routes se forment effectivement entre entrée et sortie ;
- la dynamique devient moins flash & die ;
- les paramètres scalent mieux au-delà de 50k.

## 7.5 Critères d’acceptation

- au moins une tâche anti-biais est apprise ;
- le réseau avec plasticité surpasse la baseline topologie-only ;
- les chemins renforcés sont topologiquement cohérents ;
- l’activité soutenue progresse sans instabilité globale ;
- un protocole de preuve d’apprentissage est archivé et reproductible.

---

# 8. V6 — Formation de chemins dopaminergiques, mémoire guidée robuste, reward prediction minimale

## 8.1 Intention

Une fois la preuve d’apprentissage réelle obtenue, la V6 doit consolider ce qui manque encore :
- rendre les chemins appris robustes ;
- stabiliser une mémoire guidée moins fragile ;
- commencer à prédire la valeur ou la récompense au lieu de simplement y réagir.

## 8.2 Objectifs

- formation explicite de routes préférentielles ;
- consolidation plus robuste et plus localisée ;
- meilleure résistance à l’homéostasie érosive ;
- première forme de reward prediction ou valeur locale.

## 8.3 Ce que doit prouver la V6

- les chemins formés persistent assez pour être réutilisés ;
- la récompense n’est plus seulement consommée, elle commence à être anticipée ;
- la mémoire utile résiste mieux au bruit et au temps.

---

# 9. V7 — Hiérarchie de zones et routage multi-échelle

## 9.1 Intention

Une fois l’apprentissage réel et les chemins robustes établis, il devient pertinent d’introduire une architecture régionale hiérarchique.

## 9.2 Objectifs

- micro / méso / macro-zones ;
- routage multi-échelle ;
- budgets régionaux ;
- zones spécialisées par rôle ;
- modulation descendante simple.

---

# 10. V8 — Spécialisation neuronale riche et canaux multiples

## 10.1 Intention

Après hiérarchie, on peut diversifier réellement les neurones.

## 10.2 Objectifs

- dépasser E/I ;
- introduire sous-types fonctionnels ;
- ajouter au moins un canal modulateur supplémentaire si utile ;
- préparer la mémoire active.

---

# 11. V9 — Assemblées dynamiques et mémoire de travail

## 11.1 Intention

Faire émerger des groupes réactivables qui peuvent porter une information au-delà du stimulus immédiat.

## 11.2 Objectifs

- formation d’assemblées ;
- maintien temporaire ;
- réactivation partielle ;
- compétition entre assemblées ;
- mémoire de travail active.

---

# 12. V10 — Chaînage, prédiction temporelle et proto-raisonnement

## 12.1 Intention

Passer des assemblées isolées à leurs transitions.

## 12.2 Objectifs

- apprendre A→B→C ;
- produire des prédictions temporelles ;
- maintenir un contexte interne court ;
- initier des boucles de comparaison attente / résultat.

---

# 13. V11 — Symbolisation et interface discrète

## 13.1 Intention

Avant le texte, il faut une couche discrète stable.

## 13.2 Objectifs

- vocabulaire interne minimal ;
- association motifs ↔ symboles ;
- encodeur/décodeur discret ;
- readout symbolique.

---

# 14. V12 — Boucle linguistique externe minimale

## 14.1 Intention

Relier la couche symbolique au texte externe.

## 14.2 Objectifs

- entrée textuelle minimale ;
- sortie textuelle rudimentaire ;
- micro-contexte ;
- mapping texte ↔ symboles internes.

---

# 15. V13 — Agent textuel émergent minimal

## 15.1 Intention

Construire un premier agent textuel de très bas niveau.

## 15.2 Objectifs

- mini-conversation ;
- petite mémoire récente ;
- patrons textuels appris ;
- comportement local orienté but.

---

## 16. Sprint transversal — Performance et outils

À partir de maintenant, l’axe transversal ne doit plus prioritairement être faire tourner plus vite, mais :

- maintenir la performance ;
- améliorer la visualisation ;
- renforcer les outils diagnostics ;
- automatiser les grid-search et benchmarks.

Axes transversaux recommandés :
- performance GPU continue ;
- visualisation 3D des conductances ;
- traçage des chemins ;
- check-pointing ;
- dashboards de calibration multi-échelle.

---

## 17. Gates expérimentaux corrigés

### Gate A — fin V5
Pas de poursuite si :
- l’apprentissage au-delà du biais topologique n’est pas démontré ;
- les baselines ne sont pas battues ;
- les chemins appris ne sont pas cohérents.

### Gate B — fin V9
Pas de poursuite symbolique si :
- pas d’assemblées stables ;
- pas de mémoire de travail exploitable ;
- pas de réactivation partielle crédible.

### Gate C — fin V11
Pas de boucle linguistique si :
- les symboles internes sont trop instables ;
- le readout discret est artificiel.

---

## 18. Priorité réelle

1. V5 — preuve d’apprentissage réelle, calibration multi-échelle, dynamique soutenue ;
2. V6 — chemins dopaminergiques robustes, mémoire guidée robuste, reward prediction minimale ;
3. V7 — hiérarchie de zones ;
4. V8 — spécialisation riche ;
5. V9 — assemblées et mémoire de travail ;
6. V10 — séquences et proto-raisonnement ;
7. V11 — symbolisation ;
8. V12 — boucle linguistique externe ;
9. V13 — agent textuel minimal.

---

## 19. Conclusion

La V4 a donné au projet son infrastructure d’apprentissage scalable.

Mais elle n’a pas encore donné :
- la preuve formelle d’apprentissage ;
- la preuve de formation de chemins ;
- une dynamique soutenue crédible ;
- une calibration multi-échelle robuste.

Ces points doivent être traités avant toute montée vers :
- hiérarchie ;
- spécialisation ;
- assemblées ;
- symbolisation ;
- texte.
