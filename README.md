# vague-search

An approximate search engine (project for the text-mining course).

## Authors

- Nicolas Mémeint
- Tom Méchineau

## Features

- Fast
- Low memory footprint
- Compatible with **any** valid UTF-8 words

## Pre-requisites

- Latest (stable) Rust toolchain
  - See [the Rust website](https://www.rust-lang.org/learn/get-started)
- *optional* A POSIX-compatible OS
  - If your OS is Windows or another non-POSIX OS, the entire compiled dictionary will be loaded,
   instead of loading it dynamically via the `mmap` syscall

## Usage

An example list of shell commands to build and run the project:

```bash
# Build the binaries in the current folder
./build.sh

# Compile the dictionary
./TextMiningCompiler /path/to/words.txt /path/to/dict.bin

# Search words in the dictionary
echo "approx 0 test" | ./TextMiningApp /path/to/dict.bin
echo "approx 1 test" | ./TextMiningApp /path/to/dict.bin
echo "approx 2 test" | ./TextMiningApp /path/to/dict.bin
echo "approx 0 test\napprox 1 test\napprox 2 test\napprox 3 test\napprox 4 test" | ./TextMiningApp /path/to/dict.bin
cat test.txt | ./TextMiningApp /path/to/dict.bin
```

## Documentation

```bash
cargo doc --workspace --open
```

## Tests

```bash
cargo test --workspace
```

## Questions

### 1. Décrivez les choix de design de votre programme.  
Notre programme est codé en Rust.  
A la fois pour permettre une optimisation accrue de l’utilisation de la mémoire et des opérations. Tout en gardant un seuil de sécurité notamment en comparaison au C.  

Nous avons décidé de partir sur une structure de donnée type Patricia trie originellement pour créer le dictionnaire puis une version compilée de celui-ci pour l’application.
Plus de détails sur la version compilée dans la question 4. 

### 2. Listez l’ensemble des tests effectués sur votre programme (en plus des units tests).
#### Patricia Trie
```bash
running 11 tests
test patricia_trie::tests::delete_combination ... ok
test patricia_trie::tests::delete_not_existing ... ok
test patricia_trie::tests::empty_creation ... ok
test patricia_trie::tests::inner_search ... ok
test patricia_trie::tests::insert_continuation_word ... ok
test patricia_trie::tests::insert_in_already_word ... ok
test patricia_trie::tests::insert_multiple_different_words ... ok
test patricia_trie::tests::insert_one_word ... ok
test patricia_trie::tests::multiple_insert_inner_delete ... ok
test patricia_trie::tests::simple_insert_delete ... ok
test patricia_trie::tests::simple_search ... ok
```

#### Compiled Trie
```bash
running 17 tests
test trie::from_trie::test::test_from_all_naive ... ok
test trie::from_trie::test::test_from_all_patricia ... ok
test trie::from_trie::test::test_from_empty ... ok
test trie::from_trie::test::test_from_all_ranges ... ok
test trie::from_trie::test::test_heuristic_all_patricia ... ok
test trie::from_trie::test::test_from_mixed ... ok
test trie::from_trie::test::test_heuristic_all_simple ... ok
test trie::from_trie::test::test_heuristic_empty ... ok
test trie::from_trie::test::test_heuristic_mixed ... ok
test trie::from_trie::test::test_heuristic_compact_ranges ... ok
test trie::from_trie::test::test_heuristic_partial_ranges ... ok
test trie::trie_node::test::test_patricia_nb_siblings_and_str_len ... ok
test utils::test::test_as_bytes_bytes ... ok
test utils::test::test_as_bytes_i32 ... ok
test trie::trie_node::test::test_trie_nodes_correct_type ... ok
test utils::test::test_as_bytes_vec_i32 ... ok
test utils::test::test_char_dist ... ok
```

#### Search
```bash
running 11 tests
test layer_stack::test::test_fetch_last_3_layers ... ok
test layer_stack::test::test_one_layer ... ok
test search_approx::test::test_compute_layer_abaca_alabama ... ok
test search_approx::test::test_compute_layer_abcdef_badcfe ... ok
test search_approx::test::test_compute_layer_alabama_abaca ... ok
test search_approx::test::test_compute_layer_kries_crise ... ok
test search_approx::test::test_compute_layer_one_layer_same_char ... ok
test search_approx::test::test_compute_layer_one_layer_same_diff_char ... ok
test search_approx::test::test_compute_layer_one_layer_same_not_first_char ... ok
test search_exact::test::mixed_search ... ok
test layer_stack::test::test_many_layers ... ok
```

#### Approximation search tests
> Example query:
>```bash
> echo "approx 0 alabama" | .\TextMiningApp.exe dict.bin
>Reading compressed dictionary...
>Listening for queries in stdin...
>[{"word":"alabama","freq":546707,"distance":0}]
>```

> Comparison:  

```bash 
echo "approx 5 alabama" 
```  
Us:
- 0.6s
- ~20Mo

Ref:
- 3.0s
- ~1.2Go


### 3. Avez-vous détecté des cas où la correction par distance ne fonctionnait pas (même avec une distance élevée)?
Le seul cas ou cela ne fonctionnerait pas est si le mot sort de l'UTF-8, par exemple en UTF-16.  
Notre programme gèrant extrèment bien les approximations toute correction en UTF-8 est possible hors de ce dernier cas. 

### 4. Quelle est la structure de données que vous avez implémentée dans votre projet, pourquoi?

Nous avons implémenté un **Patricia trie** et une version compilé de ce dernier que customisé que nous avons appelé **Compiled Trie**.  
Le Patricia trie normal est utilisé pour construire le trie à partir de la liste des mots fournis au moment du compilateur.
![Patricia Trie](https://upload.wikimedia.org/wikipedia/commons/thumb/a/ae/Patricia_trie.svg/1200px-Patricia_trie.svg.png)

Puis pour optimiser l’application nous avons mis au point un Compiled Trie prenant beaucoup moins de places en mémoire qu’en conditions habituelles avec notamment un mélange de plusieurs types de noeuds.
- **Naive Node:** noeud contenant une lettre et la fréquence si mot
- **Range Node:** noeud contenant une suite de un caractère en ordre lexicale permettant de compacter un grand nombre de naives nodes un seul noeud. Chaque caractère dans la suite à ainsi la possiblité d'avoir des enfants et d'être un mot.
- **Patricia Node:** noeud commun d'un Patricia Trie contenant une string et la fréquence optionnelle du mot.  

L’accès est à la fois bien plus rapide et engendre un gain de mémoire par deux au minimum.

![Compiled Trie](https://i.imgur.com/5YWb21b.png)

### 5. Proposez un réglage automatique de la distance pour un programme qui prend juste une chaîne de caractères en entrée, donner le processus d’évaluation ainsi que les résultats.
Deux choix sont possibles, une distance fixe ou une distance fluctuante. 

La distance fixe permettrait d'uniformiser les résultats et les recherches. Mais réduirait la généralisation.

La distance changeante pourrait permettre de mieux s'adapter au contexte du mot et aux options disponibles. On pourrait par exemple prendre une distance grande en fonction de la taille du mot avec un certain logarithme.
Il faudrait évaluer cela par des humains et comparés à ce qu'un humain attendrait. La métrique serait ainsi le nombre de résultats utiles.  
Côté utilisateur, afin de confirmer la qualité de l'algorithme, un panel d'humains votant sur la métrique défini plus haute aiderait à l'évaluation.
 
### 6. Comment comptez vous améliorer les performances de votre programme?
Nous avons déjà optimisé des meilleurs façons que nous le pouvions mais il existe encore peut être des moyens d’accélérer notamment la création du dictionnaire compilé avec une meilleure complexité sur l’insertion et le passage de Patricia trie normal à Patricia trie compilé. Ceci dis, cela concerne exclusivement le programme compilateur.

Au regard de nos résultats sur le programme principale, la vitesse est un des points à améliorer pour des approximations de petites distances. Autrement, nous battons le programme référence que ce soit au niveau mémoire mais aussi vitesse sur les grandes distances d'approximations.  

#### Low distance approximation
|          |  Reference  |  Vague search |
|----------|:-----------:|:-------------:|
| Speed    |   &#9745;   |   &#9744;     |
| Memory usage  |   &#9744;   |   &#9745;     |

#### High distance approximation
|          |  Reference  |  Vague search |
|----------|:-----------:|:-------------:|
| Speed    |   &#9744;   |   &#9745;     |
| Memory usage  |   &#9744;   |   &#9745;     |

### 7. Que manque-t-il à votre correcteur orthographique pour qu’il soit à l’état de l’art
Une utilisation de Machine Learning aiderait à prendre une distance approprié en fonction des mots rentrés. Avec bien entendu un algorithme entrainé préalablement sur un set de données s'approchant du dictionnaire évalué.  

Il faudrait aussi pouvoir découper une phrase en plusieurs tokens et en garder le contexte pour accroitre la précision du correcteur individuel. Ce qui n'est pas forcément possible avec un trie.  
Le mot recherché dépend en effet des mots autour de celui-ci.

Autrement en terme exclusivement de technique, notre correcteur orthographique utilise les techniques et optimisations les plus évoluées de nos jours.