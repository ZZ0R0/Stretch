# Roadmap principale — révision post-V3
## Versioning sans versions intermédiaires

## 1. Objet du document

Ce document remplace la version précédente de la roadmap post-V3 avec une règle explicite :

> **aucune version intermédiaire de type V3.x, V4.x, etc.**

Lorsqu’un palier technique supplémentaire est nécessaire pour valider de nouvelles briques, on :
- **élargit la version supérieure** ;
- ou **on crée directement une version majeure suivante**.

Les validations partielles deviennent donc :
- des **gates** ;
- des **jalons internes** ;
- des **sous-phases de travail**,

mais **jamais** des numéros de version intermédiaires.

---

## 2. Base considérée comme acquise

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
- formalisation mathématique complète.

### V2 — activité endogène régulée
- partitionnement spatial ;
- PID ;
- pacemakers ;
- consolidation mémoire structurelle ;
- activité auto-entretenue.

### V3 — transition vers des dynamiques internes
- neurones excitateurs / inhibiteurs ;
- propagation signée ;
- PID indirect ;
- STDP ;
- budget synaptique compétitif ;
- passage à 500k nœuds sur CPU parallèle ;
- substrat moins homogène et plus auto-organisé.

---

## 3. Diagnostic post-V3

La V3 a validé un vrai saut qualitatif, mais elle laisse ouverts plusieurs verrous structurants :

- oscillations émergentes encore à confirmer strictement ;
- consolidation encore trop rapide sous fort fond d’activité ;
- STDP encore trop symétrique ;
- absence de dopamine ;
- absence de récompense ;
- absence de traces d’éligibilité ;
- absence d’interface d’entrée et de sortie ;
- absence de hiérarchie régionale ;
- différenciation neuronale encore limitée ;
- distance encore importante avant une symbolisation ou une boucle textuelle.

La conséquence principale est simple :

> la suite du projet est plus difficile que prévu initialement

et la roadmap doit être allongée.

---

## 4. Nouvelle règle de progression

La roadmap ne doit plus être pensée comme une chaîne trop compacte du type :

```text
compétition → hiérarchie → mémoire → assemblées → symboles → texte
```

mais comme une montée par blocs mieux ordonnés :

```text
V3 acquis
→ V4 dopamine / reward / éligibilité / I/O minimales
→ V5 mémoire guidée et compétition réelle
→ V6 hiérarchie de zones
→ V7 spécialisation riche
→ V8 assemblées et mémoire de travail
→ V9 séquences et prédiction
→ V10 symbolisation
→ V11 boucle linguistique externe minimale
→ V12 agent textuel émergent minimal
```

---

## 5. Nouvelle roadmap à partir de V4

```text
V0   substrat dynamique minimal                    acquis
V1   espace 3D non-grid                            acquis
V2   régulation locale et activité endogène        acquis
V3   inhibition, STDP, PID indirect                acquis / consolidé

V4   dopamine, reward, éligibilité, I/O minimales
V5   mémoire sélective guidée + compétition locale réelle
V6   hiérarchie de zones et régulation multi-échelle
V7   spécialisation neuronale riche et canaux multiples
V8   assemblées dynamiques et mémoire de travail
V9   chaînage, prédiction temporelle et proto-raisonnement
V10  symbolisation et interface discrète
V11  boucle linguistique externe minimale
V12  agent textuel émergent minimal
```

---

# 6. V4 — Dopamine, reward, éligibilité, entrées/sorties minimales

## 6.1 Intention

La V4 doit faire franchir au projet un cap fondamental :

passer d’un substrat auto-organisé à un substrat qui peut commencer à apprendre **en fonction d’un objectif**.

La V4 absorbe aussi les validations post-V3 qui auraient sinon donné lieu à une pseudo-version intermédiaire.

## 6.2 Objectifs

- finaliser les points ouverts de stabilité post-V3 dans le cadre de la V4 ;
- introduire un canal dopaminergique minimal ;
- introduire un reward externe simple ;
- introduire des traces d’éligibilité ;
- introduire une entrée minimale structurée ;
- introduire une sortie minimale interprétable.

## 6.3 Transformations majeures

### A. Stabilisation post-V3 intégrée à la V4
La V4 doit inclure, comme prérequis internes :
- validation ou réfutation propre des oscillations émergentes ;
- recalibration E/I si nécessaire ;
- STDP asymétrique si la V3 symétrique est insuffisante ;
- réduction supplémentaire de la consolidation parasite.

### B. Voies dopaminergiques minimales
- niveau tonique ;
- bursts ;
- dips ;
- modulation de la STDP ;
- gating de consolidation.

### C. Traces d’éligibilité
- trace locale par arête ;
- décroissance temporelle ;
- couplage eligibility × reward.

### D. Entrée minimale
- sous-graphe d’entrée ;
- patterns simples ;
- séparation nette entre données externes et fond interne.

### E. Sortie minimale
- sous-graphe de sortie ;
- lecture de décision simple ;
- readout comportemental élémentaire.

## 6.4 Ce que la V4 doit prouver

- le reward modifie réellement ce qui est appris ;
- l’entrée n’est plus une simple injection arbitraire ;
- la sortie est interprétable ;
- la plasticité devient partiellement guidée ;
- les derniers doutes majeurs post-V3 sont levés ou tranchés dans le cadre de la V4.

## 6.5 Critères d’acceptation

- STDP seule vs STDP+reward produit des apprentissages différents ;
- deux patterns d’entrée distincts mènent à deux réponses distinctes ;
- une tâche simple obtient une performance au-dessus du hasard ;
- la dopamine module l’apprentissage sans devenir un nouveau PID masqué ;
- les oscillations émergentes ont été soit validées, soit proprement reclassées comme non nécessaires pour la suite immédiate.

---

# 7. V5 — Mémoire sélective guidée et compétition locale réelle

## 7.1 Intention

Après introduction de reward et d’éligibilité, la mémoire doit devenir réellement **sélective et utile**.

## 7.2 Objectifs

- consolider en fonction de la valeur ;
- rendre la compétition locale réellement discriminante ;
- limiter fortement la mémoire parasite ;
- raffiner la STDP asymétrique.

## 7.3 Transformations majeures

- consolidation sous reward ;
- gating événementiel ;
- seuils adaptatifs de consolidation ;
- compétition locale renforcée ;
- winner-take-all local plus robuste ;
- sélection des motifs utiles.

## 7.4 Ce que doit prouver la V5

- le système n’apprend plus seulement ce qui est fréquent ;
- il apprend davantage ce qui est pertinent ou récompensé ;
- la mémoire devient plus rare, plus informative, plus stable.

## 7.5 Critères d’acceptation

- baisse nette de la consolidation parasite ;
- motifs gagnants plus localisés ;
- amélioration mesurable sur tâche récompensée ;
- stabilité globale conservée.

---

# 8. V6 — Hiérarchie de zones et régulation multi-échelle

## 8.1 Intention

Reconstruire l’architecture régionale sur une base désormais orientée vers l’apprentissage et la valeur.

## 8.2 Objectifs

- micro / méso / macro-zones ;
- contrôle multi-échelle ;
- budgets régionaux ;
- modulation descendante ;
- répartition fonctionnelle plus claire.

## 8.3 Transformations majeures

- zones hiérarchiques ;
- métriques régionales ;
- modulation top-down ;
- contraintes énergétiques ;
- re-partitionnement partiel si nécessaire.

## 8.4 Ce que doit prouver la V6

- certaines zones intègrent ;
- certaines relaient ;
- certaines arbitrent ;
- la hiérarchie améliore stabilité et lisibilité.

## 8.5 Critères d’acceptation

- rôles régionaux mesurables ;
- coordination multi-zones ;
- spécialisation régionale visible ;
- compatibilité avec reward et I/O.

---

# 9. V7 — Spécialisation neuronale riche et canaux multiples

## 9.1 Intention

Passer d’une différenciation principalement E/I à une diversité neuronale réellement fonctionnelle.

## 9.2 Objectifs

- enrichir `NeuronType` ;
- introduire plusieurs constantes de temps et rôles ;
- ajouter au moins un canal modulateur supplémentaire si nécessaire ;
- préparer mémoire de travail et assemblées.

## 9.3 Transformations majeures

Familles possibles :
- relais rapides ;
- intégrateurs lents ;
- neurones mémoire ;
- oscillatoires ;
- sortie ;
- interneurones rapides/lents.

Canaux possibles :
- dopamine ;
- saillance / attention ;
- modulation lente tonique.

## 9.4 Ce que doit prouver la V7

- la diversité neuronale améliore réellement le système ;
- certaines fonctions deviennent dépendantes de certains types ;
- la complexité ajoutée reste contrôlable.

## 9.5 Critères d’acceptation

- ablations parlantes ;
- spécialisation régionale accrue ;
- gains mesurables sur mémoire, stabilité ou décision.

---

# 10. V8 — Assemblées dynamiques et mémoire de travail

## 10.1 Intention

Faire émerger des groupes fonctionnels identifiables, réactivables et compétitifs.

## 10.2 Objectifs

- formation d’assemblées ;
- maintien temporaire ;
- dissolution ;
- réactivation partielle ;
- mémoire de travail active.

## 10.3 Transformations majeures

- détection d’assemblées ;
- maintien récurrent ;
- inhibition inter-assemblées ;
- replay local ;
- compétitions entre motifs.

## 10.4 Ce que doit prouver la V8

- un motif peut être maintenu après le stimulus ;
- un motif partiel peut rappeler le motif complet ;
- plusieurs assemblées peuvent entrer en compétition.

## 10.5 Critères d’acceptation

- assemblées détectables ;
- mémoire de travail > quelques dizaines de ticks ;
- réactivation partielle mesurable ;
- winner-take-all non trivial.

---

# 11. V9 — Chaînage, prédiction temporelle et proto-raisonnement

## 11.1 Intention

Passer des assemblées à leurs transitions.

## 11.2 Objectifs

- apprendre A→B→C ;
- produire des prédictions temporelles ;
- maintenir un contexte interne court ;
- tester des boucles d’évaluation simples.

## 11.3 Transformations majeures

- transitions entre assemblées ;
- chaînage ;
- prédiction ;
- erreurs locales de prédiction ;
- premiers scénarios séquentiels.

## 11.4 Ce que doit prouver la V9

- séquences rejouables ;
- alternatives départageables ;
- contexte simple conservé ;
- début de proto-raisonnement séquentiel.

## 11.5 Critères d’acceptation

- réussite sur chaînes simples ;
- prédiction meilleure que le hasard ;
- stabilité séquentielle multi-run.

---

# 12. V10 — Symbolisation et interface discrète

## 12.1 Intention

Avant le texte, il faut une couche de représentations discrètes manipulables.

## 12.2 Objectifs

- construire un petit vocabulaire interne ;
- associer motifs / assemblées / chaînes à des unités discrètes ;
- manipuler de petites entrées / sorties discrètes.

## 12.3 Transformations majeures

- tokenisation interne ;
- dictionnaire de motifs ;
- encodeur/décodeur discret ;
- catégorisation élémentaire.

## 12.4 Ce que doit prouver la V10

- existence de symboles internes stables ;
- relations simples entre symboles ;
- readout discret fiable.

## 12.5 Critères d’acceptation

- quelques symboles robustes ;
- rappel symbolique simple ;
- transitions symboliques simples réussies.

---

# 13. V11 — Boucle linguistique externe minimale

## 13.1 Intention

Séparer la symbolisation interne de la première interface linguistique externe.

## 13.2 Objectifs

- entrée texte très limitée ;
- sortie texte rudimentaire ;
- mapping texte ↔ symboles internes ;
- micro-contexte.

## 13.3 Transformations majeures

- dictionnaire externe minimal ;
- tokenisation textuelle simple ;
- boucle d’échange très contrainte.

## 13.4 Ce que doit prouver la V11

- lien réel entre texte externe et états internes ;
- réponses sur petit vocabulaire ;
- conservation du sujet immédiat.

## 13.5 Critères d’acceptation

- micro-échanges simples ;
- rappel immédiat ;
- cohérence locale rudimentaire.

---

# 14. V12 — Agent textuel émergent minimal

## 14.1 Intention

Construire un premier agent textuel de très bas niveau à partir des briques précédentes.

## 14.2 Objectifs

- mini-conversation ;
- petite mémoire récente ;
- patrons textuels appris ;
- comportement local orienté but.

## 14.3 Ce que doit prouver la V12

- plusieurs tours très courts ;
- maintien d’une consigne locale ;
- rappel très récent ;
- comportement supérieur à un simple réflexe direct.

## 14.4 Critères d’acceptation

- mini-dialogues simples ;
- cohérence locale sur quelques tours ;
- apprentissage incrémental de patrons.

---

## 15. Sprint transversal — Accélération GPU

L’accélération GPU reste un sujet transversal.

### Déclenchement conseillé
- dès V4 si reward + I/O + éligibilité rendent les sweeps trop lents ;
- impératif au plus tard entre V5 et V7 si la cible reste ≥500k nœuds avec instrumentation riche.

### Portée cible
- propagation ;
- STDP ;
- traces d’éligibilité ;
- métriques bulk ;
- rendu instancié ;
- certains calculs hiérarchiques si utile.

---

## 16. Gates expérimentaux corrigés

### Gate A — fin V4
Pas de poursuite si :
- reward n’influe pas réellement sur l’apprentissage ;
- entrée/sortie minimales restent arbitraires ;
- points ouverts post-V3 non tranchés rendent les résultats ambigus.

### Gate B — fin V8
Pas de poursuite symbolique si :
- pas d’assemblées stables ;
- pas de mémoire de travail exploitable ;
- pas de réactivation partielle crédible.

### Gate C — fin V10
Pas de boucle textuelle si :
- les symboles internes sont trop instables ;
- le readout discret est artificiel ou non réutilisable.

---

## 17. Priorité réelle

L’ordre stratégique corrigé devient :

1. **V4** — dopamine, reward, éligibilité, I/O minimales ;
2. **V5** — mémoire guidée et compétition réelle ;
3. **V6** — hiérarchie de zones ;
4. **V7** — spécialisation riche ;
5. **V8** — assemblées et mémoire de travail ;
6. **V9** — séquences et prédiction ;
7. **V10** — symbolisation ;
8. **V11** — boucle linguistique externe ;
9. **V12** — agent textuel minimal.

---

## 18. Conclusion

La correction post-V3 change fortement la suite du projet.

Le projet a maintenant besoin, beaucoup plus tôt que prévu, de :

- **dopamine**
- **reward**
- **éligibilité**
- **entrée**
- **sortie**

et ces briques doivent être absorbées directement par **V4**, sans créer de version intermédiaire.

La roadmap corrigée devient donc :

```text
V4   dopamine, reward, éligibilité, I/O minimales
V5   mémoire sélective guidée + compétition locale réelle
V6   hiérarchie de zones et régulation multi-échelle
V7   spécialisation neuronale riche et canaux multiples
V8   assemblées dynamiques et mémoire de travail
V9   chaînage, prédiction temporelle et proto-raisonnement
V10  symbolisation et interface discrète
V11  boucle linguistique externe minimale
V12  agent textuel émergent minimal
```

Cette version respecte la contrainte de versioning :
**pas de V3.x, pas de versions intermédiaires, uniquement des versions supérieures.**
