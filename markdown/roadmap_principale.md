# Roadmap principale — révision à partir de V3
## Substrat cognitif spatio-temporel

## 1. Objet

Ce document met à jour la roadmap à partir de **V3** en tenant compte de :

- la roadmap actuelle ;
- les aboutissements réels de la V2 ;
- les instabilités et limites identifiées après V2 ;
- la vision post-V2.

Les versions **V0**, **V1** et **V2** sont considérées comme acquises et ne constituent plus le centre de la planification.

L’objectif de cette révision est de remplacer une progression trop directe par une progression **plus réaliste**, **plus sûre**, et **mieux séquencée** vers un agent textuel émergent.

---

## 2. Base acquise avant V3

### V0 — substrat dynamique minimal
- propagation locale ;
- dissipation ;
- traces ;
- plasticité locale ;
- corridors préférentiels.

### V1 — substrat spatial 3D non-grid
- graphe spatial 3D ;
- topologies KNN / radius ;
- indexation KD-tree ;
- calibration 3D ;
- visualisation 3D ;
- formalisation mathématique complète.

### V2 — activité endogène régulée
- partitionnement spatial en zones ;
- régulation PID ;
- pacemakers ;
- consolidation mémoire structurelle ;
- activité auto-entretenue ;
- fonctionnement stable à 50k nœuds.

---

## 3. Réalité technique après V2

La V2 a résolu le blocage fondamental de V1 : l’absence d’activité endogène.  
Le système possède désormais un fond d’activité stable, des oscillations imposées et une mémoire structurelle durable. Cela constitue une avancée décisive. Cependant, la V2 a aussi révélé des verrous qui empêchent toute montée réaliste vers des comportements cognitifs riches.

### 3.1 Limites désormais avérées

- la consolidation n’est pas sélective et tend à se généraliser à presque tout le graphe ;
- l’activité maintenue par le PID est trop homogène spatialement ;
- le PID reste trop direct et court-circuite la propagation autonome ;
- les oscillations sont encore imposées par les pacemakers, non émergentes ;
- il n’existe pas encore d’inhibition inter-neuronale ;
- la plasticité reste corrélative et non causale ;
- il n’existe ni assemblées compétitives, ni mémoire de travail, ni apprentissage séquentiel ;
- la scalabilité temps réel reste limitée par le CPU.

---

## 4. Conséquence sur la roadmap

La roadmap précédente était trop compacte à partir de V3.  
Elle supposait que l’on pourrait enchaîner presque directement :

- hiérarchie régionale ;
- spécialisation neuronale ;
- mémoire multi-système ;
- assemblées ;
- symbolisation ;
- agent textuel.

Les résultats V2 montrent que ce saut est trop ambitieux.  
Il faut insérer des paliers supplémentaires pour résoudre d’abord :

1. la compétition ;
2. la causalité temporelle ;
3. l’auto-entretien par propagation ;
4. la sélectivité de la mémoire ;
5. la montée en échelle.

La roadmap corrigée devient donc :

```text
V0  substrat dynamique minimal
V1  espace 3D non-grid
V2  régulation locale et activité endogène

V3  inhibition, STDP, PID indirect
V4  sélectivité mémoire et compétition locale
V5  hiérarchie de zones et régulation multi-échelle
V6  spécialisation neuronale riche
V7  mémoire multi-système
V8  assemblées dynamiques et mémoire de travail
V9  chaînage, séquences et proto-raisonnement
V10 symbolisation et interface discrète
V11 agent textuel émergent minimal
```

La V8 n’est donc plus l’objectif terminal de numérotation.

---

# 5. V3 — Inhibition, STDP, PID indirect

## 5.1 Intention

Transformer la V2 d’un système **régulé depuis l’extérieur du graphe** en un système dont la dynamique commence à émerger des interactions entre types neuronaux.

La V3 est la vraie transition entre :
- **activité maintenue par contrôle**
et
- **activité maintenue par structure interne**.

## 5.2 Objectifs

- introduire l’inhibition inter-neuronale ;
- rendre la plasticité temporellement causale ;
- transformer le PID en mécanisme indirect ;
- réduire l’homogénéité spatiale ;
- faire émerger les premières oscillations non purement imposées.

## 5.3 Transformations majeures

### A. Neurones inhibiteurs
Ajout d’une proportion contrôlée de neurones inhibiteurs, cible initiale ~20%.

Effets attendus :
- contraste spatial ;
- compétition locale ;
- prémices de winner-take-all ;
- base des oscillations E/I émergentes.

### B. PID indirect
Le PID ne doit plus injecter directement dans `activation`.

Il doit ajuster localement :
- le seuil ;
- le gain de propagation ;
- l’excitabilité ;
- éventuellement le budget énergétique de zone.

Le réseau doit redevenir responsable de sa propre activité.

### C. STDP
Remplacement ou extension de la plasticité corrélative par une plasticité dépendante du timing.

Effets attendus :
- causalité directionnelle ;
- apprentissage de séquences ;
- différenciation de A→B et B→A.

### D. Pacemakers adoucis
Les pacemakers deviennent des outils de guidage ou d’amorçage, pas la source principale d’oscillation.

### E. Première sélectivité mémoire
Début de normalisation hebbienne / budget sortant / consolidation compétitive.

## 5.4 Ce qui doit être livré

- neurones excitateurs / inhibiteurs ;
- STDP opérationnelle ;
- PID indirect configurable ;
- benchmarks comparaison PID direct vs indirect ;
- premiers tests d’oscillations émergentes sans pacemaker.

## 5.5 Critères d’acceptation

- au moins une forme d’oscillation émergente observable sans pacemaker ;
- premières compétitions locales reproductibles ;
- apprentissage directionnel mesurable sur séquences simples ;
- baisse sensible de la consolidation de masse ;
- activité auto-entretenue conservée malgré la réduction du rôle du PID.

## 5.6 Protocoles à produire

- cahier des charges V3 ;
- protocole E/I ;
- protocole STDP ;
- protocole PID indirect ;
- protocole oscillations émergentes ;
- benchmark CPU/GPU ciblé sur propagation + STDP.

---

# 6. V4 — Sélectivité mémoire et compétition locale

## 6.1 Intention

Stabiliser ce que la V3 aura ouvert :  
la compétition, la mémoire sélective et le contraste spatial.

Cette version existe parce que la V2 a montré que la mémoire structurelle peut devenir non informative si elle n’est pas rendue compétitive.

## 6.2 Objectifs

- empêcher la consolidation de masse ;
- imposer une vraie compétition entre connexions ;
- faire émerger des motifs localisés ;
- préparer les futures assemblées.

## 6.3 Transformations majeures

### A. Normalisation synaptique
Exemples possibles :
- budget de conductance sortant par nœud ;
- normalisation par ligne ;
- quota de connexions consolidables ;
- seuils adaptatifs de consolidation.

### B. Gating de la plasticité
La plasticité structurelle et la consolidation ne doivent plus dépendre du simple fond d’activité.

On introduit :
- seuils événementiels ;
- fenêtres de plasticité ;
- gating par intensité locale ;
- éventuellement modulation locale.

### C. Inhibition latérale
Développement du contraste et de la sélection.

### D. Détection de motifs localisés
Instrumentation dédiée pour distinguer :
- activité de fond ;
- événements structurants ;
- motifs réellement appris.

## 6.4 Ce qui doit être livré

- mécanisme de consolidation compétitive ;
- normalisation synaptique ;
- métriques de sparsité fonctionnelle ;
- cartes de motifs et chemins réellement sélectionnés.

## 6.5 Critères d’acceptation

- la mémoire structurelle reste limitée à une fraction informative du graphe ;
- les motifs appris sont localisés ;
- plusieurs patterns stimulés simultanément n’aboutissent plus à une simple superposition ;
- les premiers effets winner-take-all partiels sont visibles.

## 6.6 Protocoles à produire

- cahier des charges V4 ;
- protocole de sélectivité mémoire ;
- protocole de compétition locale ;
- protocole d’ablation inhibition ;
- benchmark de stabilité mémoire sur longues fenêtres.

---

# 7. V5 — Hiérarchie de zones et régulation multi-échelle

## 7.1 Intention

Après avoir réintroduit des dynamiques locales crédibles, il devient pertinent de reconstruire une organisation macroscopique.

La hiérarchie régionale ne doit plus être une simple partition statique pilotée par PID direct, mais une architecture multi-échelle compatible avec les dynamiques internes.

## 7.2 Objectifs

- introduire des niveaux micro / méso / macro ;
- passer d’un partitionnement figé à une organisation plus fonctionnelle ;
- différencier des régions par consignes, budgets, rythmes et rôles.

## 7.3 Transformations majeures

### A. Zones hiérarchiques
- micro-zones : dynamique locale ;
- méso-zones : coordination intermédiaire ;
- macro-zones : état global, allocation énergétique, rythmes dominants.

### B. Consignes différenciées
Chaque zone n’a plus forcément la même activité cible.

### C. Budgets métaboliques
Introduction de coûts et de contraintes par région.

### D. Régulation multi-échelle
Le contrôle global agit sur les paramètres de zones, pas sur l’activation brute.

### E. Possibilité de re-partitionnement
Au moins partiel, au fil de l’activité.

## 7.4 Ce qui doit être livré

- structure de zones hiérarchiques ;
- régulation multi-échelle ;
- métriques de spécialisation régionale ;
- instrumentation micro/méso/macro.

## 7.5 Critères d’acceptation

- des régions montrent durablement des comportements différents ;
- la hiérarchie améliore la stabilité et la lisibilité des motifs ;
- les zones ne sont plus simplement isofonctionnelles ;
- le système supporte des rythmes locaux et globaux simultanés.

## 7.6 Protocoles à produire

- cahier des charges V5 ;
- protocole de hiérarchie régionale ;
- protocole budgets métaboliques ;
- protocole de re-partitionnement ;
- benchmark de coût hiérarchique.

---

# 8. V6 — Spécialisation neuronale riche

## 8.1 Intention

Une fois inhibition, STDP, compétition et hiérarchie en place, le système peut enfin supporter une véritable différenciation neuronale utile.

## 8.2 Objectifs

- introduire plusieurs familles de neurones non redondantes ;
- faire émerger des rôles fonctionnels distincts ;
- préparer les mémoires spécialisées.

## 8.3 Types cibles

Exemples de familles :
- excitateurs ;
- inhibiteurs ;
- relais rapides ;
- intégrateurs lents ;
- neurones mémoire ;
- neurones de contrôle ;
- neurones oscillatoires ;
- neurones d’action / sortie.

## 8.4 Transformations majeures

Chaque famille peut différer selon :
- constantes de temps ;
- fatigue ;
- inhibition ;
- trace ;
- plasticité ;
- sensibilité au contrôle régional ;
- connectivité privilégiée.

## 8.5 Ce qui doit être livré

- système de types riche ;
- matrice de compatibilité inter-types ;
- instrumentation fonctionnelle par type ;
- ablations de type.

## 8.6 Critères d’acceptation

- les types ont des rôles mesurables ;
- certaines régions s’enrichissent naturellement en certains types ;
- la diversité neuronale améliore la qualité des dynamiques ;
- les premiers circuits spécialisés apparaissent.

## 8.7 Protocoles à produire

- cahier des charges V6 ;
- protocole de validation par type ;
- protocole de spécialisation régionale ;
- protocole d’ablation multi-types.

---

# 9. V7 — Mémoire multi-système

## 9.1 Intention

Passer d’une mémoire essentiellement structurelle à plusieurs régimes de mémoire distincts.

## 9.2 Objectifs

- mémoire de travail ;
- mémoire consolidée ;
- mémoire procédurale ;
- replay ;
- oubli utile.

## 9.3 Transformations majeures

### A. Mémoire de travail
Maintien temporaire d’un pattern spécifique sans stimulus permanent.

### B. Mémoire long terme
Chemins consolidés mais sélectifs.

### C. Mémoire procédurale
Séquences causales consolidées via STDP.

### D. Replay
Réactivation spontanée ou quasi-spontanée de patterns appris.

### E. Oubli utile
Éviter la saturation mémorielle.

## 9.4 Ce qui doit être livré

- architecture mémoire multi-système ;
- scheduler actif/repos/replay ;
- métriques de rappel différé ;
- métriques de maintien temporaire.

## 9.5 Critères d’acceptation

- un pattern peut être maintenu temporairement puis oublié ;
- des séquences apprises peuvent être rappelées ;
- des réactivations de replay sont observables ;
- la mémoire reste sélective et non triviale.

## 9.6 Protocoles à produire

- cahier des charges V7 ;
- protocole mémoire de travail ;
- protocole replay ;
- protocole mémoire procédurale ;
- protocole oubli.

---

# 10. V8 — Assemblées dynamiques

## 10.1 Intention

À ce stade, le système ne doit plus seulement propager et mémoriser, mais former des **assemblées** identifiables et manipulables.

## 10.2 Objectifs

- faire émerger des assemblées stables ou méta-stables ;
- permettre la compétition entre assemblées ;
- rendre possibles l’activation, la dissolution et la réactivation d’assemblées.

## 10.3 Transformations majeures

### A. Détection d’assemblées
Outils et métriques pour identifier les ensembles co-actifs.

### B. Maintien et dissolution
Règles permettant à une assemblée de persister un temps donné puis de s’éteindre.

### C. Compétition entre assemblées
Winner-take-all partiel, inhibition latérale, arbitrage.

### D. Réactivation
Une assemblée partiellement stimulée doit pouvoir se réactiver comme motif.

## 10.4 Ce qui doit être livré

- moteur d’assemblées ;
- instrumentation de motifs ;
- mesures de stabilité d’assemblée ;
- tests de réactivation partielle.

## 10.5 Critères d’acceptation

- au moins une assemblée méta-stable est détectable et reproductible ;
- deux assemblées concurrentes n’aboutissent pas toujours à la coexistence ;
- un motif partiel peut parfois rappeler l’assemblée complète ;
- les assemblées ne sont pas des artefacts de visualisation mais des structures mesurées.

## 10.6 Protocoles à produire

- cahier des charges V8 ;
- protocole d’assemblées ;
- protocole de compétition d’assemblées ;
- protocole de réactivation partielle.

---

# 11. V9 — Chaînage, séquences et proto-raisonnement

## 11.1 Intention

Une fois les assemblées établies, on peut tenter leur enchaînement et leur manipulation.

## 11.2 Objectifs

- faire suivre une assemblée par une autre ;
- apprendre des transitions ;
- commencer à maintenir un état interne opératoire ;
- introduire une forme de proto-raisonnement séquentiel.

## 11.3 Transformations majeures

### A. Chaînage d’assemblées
Assemblage A → Assemblage B → Assemblage C.

### B. Sélection de transition
Quand plusieurs suites sont possibles, le système doit arbitrer.

### C. État interne opératoire
Maintien d’un contexte très simple pendant quelques étapes.

### D. Premières boucles d’évaluation
Tester une suite, comparer le résultat à une attente simple, corriger.

## 11.4 Ce qui doit être livré

- moteur de transitions entre assemblées ;
- mesure de succès séquentiel ;
- instrumentation des chaînes internes ;
- premiers scénarios de proto-résolution.

## 11.5 Critères d’acceptation

- une séquence apprise peut être rejouée sans stimulus complet ;
- des alternatives concurrentes peuvent être départagées ;
- un contexte local peut être maintenu sur plusieurs étapes ;
- les erreurs séquentielles sont mesurables.

## 11.6 Protocoles à produire

- cahier des charges V9 ;
- protocole de séquences ;
- protocole de chaînage ;
- protocole de contexte interne ;
- protocole de proto-raisonnement.

---

# 12. V10 — Symbolisation et interface discrète

## 12.1 Intention

Il serait trop optimiste de brancher directement un agent textuel après les assemblées.  
Il faut d’abord une couche de représentation discrète exploitable.

## 12.2 Objectifs

- associer des assemblées ou séquences à des unités discrètes ;
- construire un petit vocabulaire interne ;
- permettre une interface entrée/sortie discrète simple.

## 12.3 Transformations majeures

### A. Tokenisation interne
Motifs récurrents → symboles internes.

### B. Dictionnaire de motifs
Catalogage et réidentification des motifs.

### C. Entrées discrètes
Petits alphabets, labels, classes, suites simples.

### D. Sorties discrètes
Décodage motif → symbole.

## 12.4 Ce qui doit être livré

- couche de symbolisation minimale ;
- dictionnaire de motifs ;
- encodeur/décodeur expérimental ;
- tests de séquences symboliques simples.

## 12.5 Critères d’acceptation

- le système peut manipuler un petit ensemble de symboles internes ;
- il peut apprendre quelques relations discrètes simples ;
- l’interface discrète n’est pas plaquée artificiellement sans correspondance interne.

## 12.6 Protocoles à produire

- cahier des charges V10 ;
- protocole de symbolisation ;
- protocole d’I/O discrète ;
- protocole de rappel symbolique.

---

# 13. V11 — Agent textuel émergent minimal

## 13.1 Intention

Ce n’est qu’ici qu’il devient réaliste de viser un premier agent textuel, et encore à un niveau modeste.

## 13.2 Objectifs

- micro-conversation ;
- contexte local court ;
- rappel très récent ;
- patrons textuels simples ;
- réponses brèves cohérentes.

## 13.3 Transformations majeures

### A. Entrée texte minimale
Mapping texte simple → symboles internes → assemblées.

### B. État conversationnel court
Maintien du sujet récent, très local.

### C. Sélection de réponse
À partir d’assemblées/symboles/chaînes.

### D. Décodage texte
Assemblées → symboles → sortie textuelle.

## 13.4 Ce qui doit être livré

- boucle textuelle minimale ;
- protocole de conversation courte ;
- protocole de rappel immédiat ;
- mesure de cohérence locale.

## 13.5 Critères d’acceptation

- le système peut tenir de micro-échanges simples ;
- il maintient un contexte court ;
- il rappelle une information très récente ;
- il apprend quelques patrons conversationnels locaux.

## 13.6 Protocoles à produire

- cahier des charges V11 ;
- protocole de conversation courte ;
- protocole de cohérence locale ;
- protocole d’apprentissage de patrons textuels.

---

## 14. Sprint transversal conseillé — Accélération GPU

Le besoin de passage à l’échelle devient crédible à partir de V3 et critique à partir de V5/V6.

Je recommande de traiter l’accélération comme un **sprint transversal**, pas comme une version cognitive autonome.

### Déclenchement conseillé
Entre V3 et V5, selon la charge réelle :
- si la STDP + inhibition + hiérarchie rendent le CPU trop lent ;
- si l’on veut viser 500k–1M nœuds.

### Portée
- propagation ;
- mises à jour locales ;
- STDP ;
- métriques bulk ;
- éventuellement gestion de zones hiérarchiques.

Le GPU ne doit pas devenir le cœur conceptuel de la roadmap, mais un accélérateur de viabilité expérimentale.

---

## 15. Gates expérimentaux

Trois gates doivent être assumés pour éviter une roadmap irréaliste.

### Gate A — fin V3
Sans inhibition + STDP + PID indirect fonctionnels, il ne faut pas poursuivre vers les mémoires supérieures.

### Gate B — fin V8
Sans assemblées stables, compétitives et réactivables, il ne faut pas prétendre à une symbolisation crédible.

### Gate C — fin V10
Sans symboles internes minimaux réellement manipulables, il ne faut pas lancer un agent textuel.

---

## 16. Ordre de priorité réel

Si l’on doit avancer vite mais correctement, l’ordre des urgences devient :

1. **V3** — inhibition, STDP, PID indirect ;
2. **V4** — sélectivité mémoire et compétition locale ;
3. **V5** — hiérarchie de zones ;
4. **V6** — spécialisation neuronale ;
5. **V7** — mémoire multi-système ;
6. **V8** — assemblées dynamiques ;
7. **V9** — séquences et proto-raisonnement ;
8. **V10** — symbolisation ;
9. **V11** — agent textuel minimal.

---

## 17. Conclusion

La V2 a rendu le projet viable.  
Mais elle a aussi montré que la suite était **plus difficile que prévu**.

La bonne correction de la roadmap n’est donc pas d’ajouter quelques sous-points :  
c’est de reconnaître qu’il faut désormais distinguer clairement :

- **auto-régulation**
- **compétition**
- **causalité temporelle**
- **hiérarchie**
- **spécialisation**
- **mémoire**
- **assemblées**
- **symbolisation**
- **texte**

La feuille de route corrigée à partir de V3 est donc :

```text
V3  inhibition, STDP, PID indirect
V4  sélectivité mémoire et compétition locale
V5  hiérarchie de zones et régulation multi-échelle
V6  spécialisation neuronale riche
V7  mémoire multi-système
V8  assemblées dynamiques
V9  chaînage, séquences et proto-raisonnement
V10 symbolisation et interface discrète
V11 agent textuel émergent minimal
```

C’est une trajectoire plus longue, mais beaucoup plus réaliste.
