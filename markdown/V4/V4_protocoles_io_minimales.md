# Protocoles entrées/sorties minimales — V4

## 1. Objet

Valider que la V4 possède une vraie boucle d’interaction minimale.

---

## 2. Hypothèse

Le système doit être capable de :
- recevoir un pattern structuré ;
- le transformer en dynamique interne ;
- produire une sortie lisible ;
- recevoir un reward sur cette sortie.

---

## 3. Entrée

## 3.1 Patterns minimaux
Utiliser :
- 2 classes binaires ;
- puis 3 ou 4 classes simples.

Exemples :
- patterns sparse fixes ;
- petites séquences discrètes.

Mesures :
- distance spatiale entre réponses internes ;
- répétabilité ;
- robustesse au bruit faible.

---

## 4. Sortie

## 4.1 Readout simple
Utiliser :
- 2 groupes de sortie ;
- ou 3 groupes si multi-classe.

Score :
- somme activité ;
- ou moyenne activité.

Décision :
- argmax.

Mesures :
- accuracy ;
- marge entre premier et second groupe ;
- latence de décision.

---

## 5. Boucle fermée minimale

Procédure :
1. injecter pattern ;
2. laisser vivre la dynamique ;
3. lire sortie ;
4. attribuer reward ;
5. répéter.

Attendu :
- évolution des poids orientée vers de meilleures sorties.

---

## 6. Critères de réussite

- patterns distincts → activations internes distinctes ;
- activations internes distinctes → sorties distinctes ;
- sorties distinctes → rewards distincts ;
- reward rétroagit sur l’apprentissage futur.
