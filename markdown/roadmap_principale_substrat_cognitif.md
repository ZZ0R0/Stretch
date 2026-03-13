# Roadmap principale — Évolution du substrat spatio-temporel cognitif à partir de la V0

## 1. Objet du document

Ce document définit la **roadmap principale** du projet à partir de la **V0 réalisée et validée**.

Il ne s’agit pas d’un simple planning.  
C’est un **cahier des charges de roadmap** destiné à servir de document directeur pour les versions futures.

Pour chaque version, la roadmap précise :

- le **but de la version** ;
- les **limites de la version précédente** qu’elle corrige ;
- les **features à intégrer ensemble** ;
- les **transformations structurelles du modèle** ;
- les **livrables attendus** ;
- les **critères d’acceptation** ;
- les **protocoles à rédiger** pour exécuter correctement la version.

L’objectif est d’avancer **vite**, mais sans diluer le projet en micro-versions dispersées.  
Les fonctionnalités sont donc **groupées par blocs de cohérence architecturale** afin de permettre une **refonte d’ensemble à chaque étape importante**.

---

## 2. Point de départ : état réel de la V0

La V0 est considérée comme **acquise**.

Elle a déjà validé :

- un substrat dynamique fonctionnel ;
- une propagation locale spatio-temporelle ;
- une dissipation stable ;
- des traces localisées ;
- une plasticité locale ;
- des corridors préférentiels ;
- une instrumentation exploitable ;
- une architecture modulaire réutilisable.

La V0 a également démontré :

- un apprentissage mesurable avant/après ;
- une zone stable non triviale ;
- une reproductibilité correcte ;
- un outillage suffisant pour itérer rapidement.

En revanche, la V0 reste très réductrice sur plusieurs points structurants :

- domaine encore borné par une représentation **2D** ;
- topologie encore trop simple ;
- absence de vraie morphogenèse ;
- absence de croissance / décroissance neuronale ;
- absence de typologie neuronale riche ;
- absence de niveaux de contrôle régionaux ;
- absence de mémoire hiérarchisée ;
- absence de mécanismes de spécialisation fonctionnelle ;
- absence de structuration vers des capacités cognitives de haut niveau.

---

## 3. Principes directeurs de la roadmap

### 3.1 Pas de dérive “LLM”
Le projet ne doit pas dériver vers une architecture de type LLM classique, ni vers une simple imitation d’un moteur de génération de texte basé sur apprentissage massif.

### 3.2 Priorité au substrat
Les premières versions doivent renforcer le **substrat dynamique**, pas surcharger le système avec des couches cognitives prématurées.

### 3.3 Groupement intelligent des features
Chaque version doit modifier un **bloc cohérent** du système :
- espace,
- dynamique,
- plasticité,
- spécialisation,
- mémoire,
- orchestration,
- cognition de haut niveau.

### 3.4 Vitesse d’itération
Chaque version doit produire :
- une amélioration qualitative visible ;
- des métriques nouvelles ;
- un protocole de validation dédié ;
- un gain d’expressivité du modèle.

### 3.5 Cap terminal
Le cap terminal de cette roadmap n’est pas la “conscience”.
Le cap terminal est :

> un système capable d’imiter un **niveau initial de cognition conversationnelle textuelle**, c’est-à-dire un équivalent fonctionnel d’une **V0 primitive d’assistant textuel**, obtenu à partir du substrat dynamique et non d’un LLM classique.

---

## 4. Découpage global de la roadmap

La roadmap est découpée en 8 grandes versions après V0 :

- **V1** : Passage au substrat spatial 3D non-grille
- **V2** : Morphogenèse et plasticité structurelle
- **V3** : Régulation régionale et neurones de contrôle
- **V4** : Typologie neuronale et spécialisation fonctionnelle
- **V5** : Mémoire multi-système et consolidation
- **V6** : Assemblages dynamiques et proto-réflexion
- **V7** : Apprentissage symbolique local et langage interne
- **V8** : Agent textuel émergent de niveau “assistant V0”

Ce découpage est volontairement compact.  
Il permet d’aller vite tout en gardant des paliers techniquement maîtrisables.

---

# 5. V1 — Passage au substrat spatial 3D non-grille

## 5.1 Intention

Corriger les deux blocages majeurs de la V0 :

- la **2D** ;
- la **grille**.

La V1 doit remplacer la grille 2D par un **substrat 3D abstrait**, avec une topologie adaptée au projet et non héritée d’une structure commode mais artificielle.

## 5.2 Objectif

Obtenir un milieu de propagation plus réaliste, plus flexible et plus expressif, capable de supporter ensuite :
- croissance,
- zones,
- gradients,
- anisotropie,
- architectures hétérogènes.

## 5.3 Transformations majeures

### A. Passage au domaine 3D
- positions 3D abstraites pour tous les nœuds ;
- distances 3D ;
- voisinages 3D ;
- visualisation adaptée.

### B. Abandon de la grille
Remplacement de `grid2d` par un ou plusieurs modèles adaptés :

- graphe spatial 3D sparse ;
- voisinage par rayon ;
- voisinage k-plus-proches voisins ;
- graphe hybride local + liaisons longues ;
- indexation spatiale optimisée.

### C. Support d’une géométrie non régulière
- densités variables ;
- régions plus ou moins compactes ;
- obstacles ou zones de faible conductivité ;
- anisotropie locale.

## 5.4 Ce qui doit être livré

- nouveau moteur de domaine 3D ;
- structures de données spatiales adaptées ;
- nouveaux noyaux de propagation compatibles 3D ;
- benchmarks CPU/GPU ciblés sur le domaine 3D ;
- visualisation 3D minimale ou projections multi-vues.

## 5.5 Critères d’acceptation

- la propagation 3D est stable ;
- les métriques restent interprétables ;
- les corridors émergent aussi en 3D ;
- les performances restent compatibles avec l’itération rapide ;
- la topologie n’est plus dépendante d’une grille.

## 5.6 Protocoles à produire

- cahier des charges V1 ;
- protocole de calibration du domaine 3D ;
- protocole de benchmark topologique ;
- protocole de comparaison V0 vs V1.

---

# 6. V2 — Morphogenèse et plasticité structurelle

## 6.1 Intention

Passer d’un réseau qui ajuste ses conductances à un système qui **modifie sa propre structure**.

La V1 donne un espace crédible.  
La V2 doit lui donner la capacité de :
- croître ;
- s’éroder ;
- se réorganiser ;
- créer et supprimer des liaisons ;
- créer et supprimer des nœuds.

## 6.2 Objectif

Faire émerger non plus seulement des corridors, mais une **topologie adaptative**.

## 6.3 Transformations majeures

### A. Croissance progressive de nouveaux neurones
- création locale de nœuds ;
- bourgeonnement à partir de zones actives ;
- insertion contrôlée dans le graphe spatial 3D.

### B. Décroissance / mort de neurones
- pruning des nœuds inutiles ;
- extinction sélective ;
- nettoyage des zones non fonctionnelles.

### C. Plasticité structurelle des liaisons
- création de nouvelles liaisons locales ;
- renforcement de liaisons faibles devenues pertinentes ;
- disparition de liaisons mortes.

### D. Contraintes morphogénétiques
- budgets locaux ;
- coûts de croissance ;
- stabilité structurelle ;
- prévention des explosions topologiques.

## 6.4 Ce qui doit être livré

- moteur de plasticité structurelle ;
- gestion de topologie mutante ;
- instrumentation de croissance/décroissance ;
- métriques morphologiques.

## 6.5 Critères d’acceptation

- le système crée des structures utiles, pas du bruit ;
- la croissance reste bornée et contrôlée ;
- des sous-réseaux spécialisés commencent à émerger ;
- la topologie finale dépend réellement de l’histoire d’activité.

## 6.6 Protocoles à produire

- cahier des charges V2 ;
- protocole de morphogenèse ;
- protocole d’évaluation des artefacts structurels ;
- protocole de coût/valeur de la croissance.

---

# 7. V3 — Régulation régionale et neurones de contrôle

## 7.1 Intention

Introduire des unités capables d’agir non pas seulement comme nœuds locaux, mais comme **régulateurs de voisinage**, afin de préparer l’hétérogénéité fonctionnelle du système.

C’est une étape pivot.

## 7.2 Objectif

Permettre la gestion de **zones**, de **modes de fonctionnement**, de **gains régionaux**, de **régimes d’activité**, et préparer l’arrivée de familles neuronales différentes.

## 7.3 Transformations majeures

### A. Neurones de contrôle local/régional
Types nouveaux de nœuds pouvant moduler :
- seuils des voisins ;
- excitabilité locale ;
- inhibition locale ;
- plasticité locale ;
- ouverture/fermeture de fenêtres d’apprentissage.

### B. Découpage du substrat en régions fonctionnelles émergentes ou pilotées
- clusters ;
- régions de forte mémoire ;
- régions de forte action ;
- régions de forte propagation ;
- gradients régionaux.

### C. Dynamique globale/locale
Ajout de mécanismes de régulation multi-échelle :
- signaux régionaux ;
- rétroaction locale ;
- mécanismes anti-saturation ;
- stabilisation des zones.

## 7.4 Ce qui doit être livré

- système de modulation régionale ;
- classes de neurones régulateurs ;
- instrumentation régionale ;
- vue cartographique des zones.

## 7.5 Critères d’acceptation

- des régions présentent des dynamiques différenciées ;
- les neurones de contrôle modifient réellement leurs voisins ;
- le système gagne en stabilité et en expressivité ;
- les futures familles neuronales disposent d’un terrain prêt.

## 7.6 Protocoles à produire

- cahier des charges V3 ;
- protocole de test régional ;
- protocole de mesure de modulation ;
- protocole de stabilité multi-zones.

---

# 8. V4 — Typologie neuronale et spécialisation fonctionnelle

## 8.1 Intention

Passer d’un système de nœuds presque homogènes à un système de **familles neuronales différenciées**.

## 8.2 Objectif

Introduire une division fonctionnelle minimale :

- neurones de mémoire ;
- neurones d’action ;
- neurones de réflexion ;
- neurones de relais ;
- neurones inhibiteurs ;
- neurones de régulation.

## 8.3 Transformations majeures

### A. Spécifications distinctes par type
Chaque famille peut avoir :
- seuils différents ;
- fatigue différente ;
- trace différente ;
- plasticité différente ;
- durée d’activité différente ;
- rôle différent dans la propagation.

### B. Compatibilités inter-types
- certaines liaisons privilégiées ;
- certaines inhibitions sélectives ;
- certaines spécialisations régionales.

### C. Apparition de motifs fonctionnels
Exemples :
- poches mémoire ;
- axes d’action ;
- zones de propagation lente ;
- zones de raisonnement/réverbération.

## 8.4 Ce qui doit être livré

- système de types neuronaux ;
- matrice de compatibilité des types ;
- règles de spécialisation ;
- instrumentation par type.

## 8.5 Critères d’acceptation

- les types ont des comportements réellement différenciés ;
- l’hétérogénéité améliore les capacités du système ;
- les régions spécialisées apparaissent ou se stabilisent ;
- les motifs dynamiques deviennent plus riches qu’en V3.

## 8.6 Protocoles à produire

- cahier des charges V4 ;
- protocole de validation par type ;
- protocole de spécialisation régionale ;
- protocole d’ablation fonctionnelle.

---

# 9. V5 — Mémoire multi-système et consolidation

## 9.1 Intention

Structurer enfin la mémoire de façon explicite, au lieu de rester sur une simple trace locale généralisée.

## 9.2 Objectif

Obtenir trois régimes distincts :

- mémoire de travail ;
- mémoire consolidée ;
- mémoire procédurale / routinière.

## 9.3 Transformations majeures

### A. Mémoire de travail
- réverbération temporaire ;
- maintien d’assemblages actifs ;
- compétition entre contenus actifs.

### B. Mémoire long terme
- consolidation lente ;
- traces stabilisées ;
- réactivation de motifs.

### C. Mémoire procédurale
- corridors fortement privilégiés ;
- séquences automatisées ;
- baisse du coût énergétique sur trajets appris.

### D. Consolidation hors ligne
- phases de repos ;
- replay ;
- renforcement différé ;
- oubli sélectif.

## 9.4 Ce qui doit être livré

- architecture mémoire multi-système ;
- scheduler de phases actives/repos ;
- métriques de consolidation ;
- protocoles d’apprentissage/rappel/oubli.

## 9.5 Critères d’acceptation

- distinction nette entre mémoire de travail, mémoire stable et routines ;
- amélioration du rappel après consolidation ;
- oubli utile des traces parasites ;
- réactivation fiable de motifs appris.

## 9.6 Protocoles à produire

- cahier des charges V5 ;
- protocole de rappel différé ;
- protocole de consolidation hors ligne ;
- protocole de mémoire procédurale.

---

# 10. V6 — Assemblages dynamiques et proto-réflexion

## 10.1 Intention

Faire émerger non plus seulement des trajets, mais des **assemblages temporaires capables de maintenir, transformer et comparer des états internes**.

C’est la version où apparaît une forme de **proto-réflexion**.

## 10.2 Objectif

Permettre :
- maintien temporaire d’un état ;
- comparaison de motifs ;
- sélection entre plusieurs trajectoires ;
- boucles internes de test.

## 10.3 Transformations majeures

### A. Assemblages dynamiques
- groupes temporaires de nœuds co-fonctionnant ;
- représentations internes transitoires ;
- attracteurs temporaires.

### B. Compétition et arbitrage
- plusieurs trajectoires candidates ;
- inhibition compétitive ;
- sélection de réponse.

### C. Boucles d’évaluation
- surveillance de la stabilité ;
- mesure d’écart entre état attendu et état obtenu ;
- préparation d’un futur système d’apprentissage plus abstrait.

## 10.4 Ce qui doit être livré

- moteur d’assemblages dynamiques ;
- mécanisme de compétition ;
- indicateurs de maintien/manipulation d’états ;
- instrumentation des séquences internes.

## 10.5 Critères d’acceptation

- le système maintient temporairement des structures non triviales ;
- plusieurs options internes peuvent entrer en compétition ;
- une sélection stable est observable ;
- les motifs ne sont plus de simples propagations passives.

## 10.6 Protocoles à produire

- cahier des charges V6 ;
- protocole de mémoire active ;
- protocole de compétition ;
- protocole de maintien/manipulation interne.

---

# 11. V7 — Apprentissage symbolique local et langage interne

## 11.1 Intention

Introduire un niveau de représentation manipulable sans basculer vers un LLM classique.

## 11.2 Objectif

Permettre au substrat d’apprendre des **formes discrètes**, des **motifs récurrents**, des **unités internes réutilisables**, puis de les combiner.

## 11.3 Transformations majeures

### A. Tokenisation interne émergente
- motifs dynamiques récurrents encapsulables ;
- “jetons internes” ou symboles endogènes ;
- dictionnaire de motifs.

### B. Associations motif ↔ motif
- chaînes ;
- transitions ;
- séquences ;
- compositions.

### C. Interface minimale entrée/sortie symbolique
- entrée textuelle simple ou pseudo-textuelle ;
- projection des motifs internes vers des sorties discrètes ;
- boucle de feedback.

### D. Premières capacités de mapping structurel
- correspondance entre motifs internes et unités de langage ;
- apprentissage de petits alphabets, séquences ou schémas.

## 11.4 Ce qui doit être livré

- couche de symbolisation minimale ;
- représentation des motifs stables ;
- interface I/O discrète ;
- premiers tests de séquences symboliques.

## 11.5 Critères d’acceptation

- le système peut réutiliser des motifs comme unités manipulables ;
- il peut apprendre et rappeler des séquences simples ;
- les symboles ne sont pas collés artificiellement au moteur mais émergent de motifs stabilisés ;
- les sorties sont cohérentes et mesurables.

## 11.6 Protocoles à produire

- cahier des charges V7 ;
- protocole de symbolisation ;
- protocole de séquences discrètes ;
- protocole de rappel symbolique.

---

# 12. V8 — Agent textuel émergent de niveau “assistant V0”

## 12.1 Intention

Construire un premier système textuel capable d’imiter un **niveau très initial d’assistant conversationnel**, non par corpus massif et backprop géant, mais par assemblages, mémoire, symboles internes et routines dynamiques.

## 12.2 Objectif

Obtenir un système capable de :

- gérer un petit contexte ;
- maintenir un but local ;
- produire des réponses textuelles simples ;
- rappeler des motifs ;
- enchaîner des séquences de sortie ;
- présenter une forme élémentaire de cohérence conversationnelle.

## 12.3 Ce que “niveau assistant V0” signifie ici

Cela ne signifie pas “égaler ChatGPT moderne”.

Cela signifie viser un système capable de :
- manipulation textuelle simple ;
- réponses courtes ;
- rappel local ;
- cohérence limitée mais réelle ;
- maintien d’un état conversationnel très court ;
- apprentissage incrémental de patrons textuels.

## 12.4 Transformations majeures

### A. Boucle perception texte → état interne → sortie texte
- encodeur d’entrée très simple ;
- activation d’assemblages symboliques ;
- décision de sortie ;
- émission de tokens/segments.

### B. Contexte local
- petite mémoire active conversationnelle ;
- rappel récent ;
- conservation du sujet courant sur courte fenêtre.

### C. Routines conversationnelles
- association question → motif → sortie ;
- enrichissement progressif ;
- correction par feedback.

### D. Évaluation de cohérence minimale
- stabilité du sujet ;
- non-divergence immédiate ;
- maintien d’un style de sortie contrôlé.

## 12.5 Ce qui doit être livré

- boucle textuelle minimale ;
- encodeur/décodeur textuel expérimental ;
- protocole de conversation courte ;
- tableaux de performances comportementales.

## 12.6 Critères d’acceptation

- le système tient de micro-échanges cohérents ;
- il peut rappeler une information très récente ;
- il peut apprendre quelques patrons conversationnels ;
- son comportement textuel provient du substrat et non d’un modèle statistique géant externe.

## 12.7 Protocoles à produire

- cahier des charges V8 ;
- protocole de conversation courte ;
- protocole de cohérence locale ;
- protocole d’apprentissage de patrons textuels.

---

# 13. Ce qui ne doit pas arriver dans la roadmap

## 13.1 Anti-patterns techniques
- empiler des features sans refonte de l’architecture ;
- ajouter du “langage” avant d’avoir la mémoire ;
- ajouter de la mémoire avant d’avoir la spécialisation ;
- ajouter de la spécialisation sans régulation régionale ;
- introduire trop tôt une topologie mutante sans domaine 3D robuste.

## 13.2 Anti-patterns scientifiques
- confondre activité riche et cognition réelle ;
- prendre un gain énergétique global pour une preuve d’intelligence ;
- interpréter trop vite des attracteurs comme des “pensées”.

## 13.3 Anti-patterns produit
- chercher la démo textuelle trop tôt ;
- dégrader la vitesse d’itération ;
- casser la comparabilité entre versions.

---

# 14. Livrables obligatoires à chaque version

Chaque version future devra produire systématiquement :

## 14.1 Documents
- cahier des charges dédié ;
- protocole de test dédié ;
- protocole de benchmark/performance si nécessaire ;
- note d’aboutissement de version.

## 14.2 Logiciel
- moteur modifié ;
- configs minimales ;
- scripts d’expérience ;
- export des métriques ;
- visualisation adaptée.

## 14.3 Validation
- comparaison avant/après version précédente ;
- démonstration des nouveaux comportements ;
- démonstration qu’aucune régression critique n’a été introduite.

---

# 15. Ordre de priorité réel

Si l’on doit aller vite, l’ordre des urgences est :

1. **V1** — sortir de la 2D et de la grille ;
2. **V2** — donner au système une topologie vivante ;
3. **V3** — introduire la régulation régionale ;
4. **V4** — différencier les neurones ;
5. **V5** — structurer la mémoire ;
6. **V6** — créer de la proto-réflexion ;
7. **V7** — faire émerger un langage interne ;
8. **V8** — brancher une boucle textuelle.

Cet ordre est le plus cohérent avec ta direction.

---

# 16. Recommandation stratégique immédiate

La prochaine version doit être **V1**, mais pensée dès le départ pour ne pas casser :

- la future croissance de V2 ;
- la future régionalisation de V3 ;
- la future spécialisation de V4.

Donc la V1 ne doit pas être un simple “portage 2D → 3D”.  
Elle doit être le **socle spatial définitif** sur lequel les versions suivantes pourront se brancher.

Concrètement, cela impose de concevoir dès maintenant :

- un graphe spatial 3D mutable ;
- une indexation spatiale compatible croissance ;
- une instrumentation compatible régions ;
- une visualisation compatible densités et clusters ;
- des identifiants et structures de données compatibles avec une topologie non fixe.

---

# 17. Conclusion

La bonne direction du projet n’est pas d’ajouter des “capacités” en surface.  
La bonne direction est de faire évoluer le système par **blocs architecturaux cohérents**, dans cet ordre :

```text
substrat spatial
→ morphogenèse
→ régulation régionale
→ spécialisation neuronale
→ mémoire multi-système
→ assemblages dynamiques
→ symbolisation
→ agent textuel
```

La V0 a prouvé que le cœur expérimental fonctionne.  
La roadmap ci-dessus permet maintenant de transformer ce cœur en un système progressivement plus riche, plus structuré, et à terme capable d’imiter un premier niveau de cognition conversationnelle sans changer de paradigme.
