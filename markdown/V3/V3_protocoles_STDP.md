# Protocoles STDP — V3

## 1. Objet

Valider que la plasticité V3 devient réellement temporellement causale.

---

## 2. Hypothèse

Si A active systématiquement B avec un décalage temporel positif, alors :

- l’arête A→B doit être renforcée ;
- l’arête B→A ne doit pas l’être au même niveau.

---

## 3. Expériences

## 3.1 Expérience simple A→B

Procédure :
1. sélectionner deux nœuds ou deux petits sous-ensembles A et B ;
2. injecter A au tick t ;
3. injecter B au tick t + delta ;
4. répéter N fois.

Mesures :
- Δw(A→B)
- Δw(B→A)

Attendu :
- Δw(A→B) > 0
- Δw(B→A) <= Δw(A→B)

---

## 3.2 Inversion B→A

Reprendre le protocole inverse.

Attendu :
- renforcement inverse si l’ordre est inversé.

---

## 3.3 Sweep sur Delta_t

Faire varier Delta_t.

Mesurer :
- courbe de renforcement ;
- courbe d’affaiblissement.

Attendu :
- profil exponentiel décroissant plausible.

---

## 4. Indicateurs de réussite

- asymétrie mesurable des poids ;
- sensibilité correcte à Delta_t ;
- pas d’explosion globale des conductances ;
- capacité à apprendre une petite séquence.
