# Protocoles de stabilité — V3

## 1. Objet

Définir les tests permettant de vérifier que la V3 ne dégénère ni en :

- extinction triviale ;
- saturation globale ;
- bruit incohérent ;
- réseau figé par inhibition excessive ;
- contrôle encore trop dominant par PID.

---

## 2. Questions de stabilité

1. L’activité auto-entretenue existe-t-elle encore quand le PID devient indirect ?
2. L’inhibition empêche-t-elle la saturation sans tuer tout le réseau ?
3. La STDP reste-t-elle bornée ?
4. La sélectivité mémoire stabilise-t-elle le réseau ou le détruit-elle ?
5. Les oscillations émergentes restent-elles localisées et stables ?

---

## 3. Campagnes de test

## 3.1 Test A — zone stable globale

Faire varier :
- proportion inhibitrice ;
- gain inhibiteur ;
- Kp, Ki, Kd ;
- projections PID → seuil/gain/excitabilité ;
- A_plus, A_minus, tau_plus, tau_minus ;
- mécanisme de sélectivité mémoire.

Mesures :
- énergie totale ;
- fraction de nœuds actifs ;
- fraction d’inhibiteurs actifs ;
- temps de survie de l’activité ;
- nombre de zones mortes.

Attendu :
- région stable exploitable entre extinction et saturation.

---

## 3.2 Test B — inhibition

Comparer :
- 0% inhibiteurs
- 10%
- 20%
- 30%

Mesures :
- contraste spatial ;
- entropie de l’activité ;
- taille des motifs actifs ;
- oscillations observées.

Attendu :
- 20% environ doit produire un meilleur compromis que 0%.

---

## 3.3 Test C — PID indirect

Comparer :
- PID direct V2
- PID indirect sur seuil
- PID indirect sur gain
- PID indirect mixte

Mesures :
- activité moyenne de zone ;
- capacité d’auto-entretien ;
- homogénéité spatiale ;
- sélectivité mémoire.

Attendu :
- le PID indirect doit réduire l’homogénéité sans faire mourir le réseau.

---

## 3.4 Test D — STDP bornée

Mesures :
- distribution des poids ;
- variance des poids ;
- proportion d’arêtes saturées à w_max ;
- proportion à w_min ;
- stabilité des distributions sur longues fenêtres.

Attendu :
- pas de collapse complet vers w_min ou w_max.

---

## 4. Indicateurs d’échec

Le système V3 est instable si :
- plus de 90% des nœuds restent actifs durablement ;
- moins de 1% des nœuds restent actifs hors stimulus ;
- les poids saturent massivement ;
- l’inhibition tue toute dynamique ;
- le PID indirect ne laisse aucune autonomie au réseau.
