## Handling of Marker Packs

There's two types of Marker Packs:
1. Xml Packs (zipped with taco extension or non-zipped in a folder)
2. Json packs (single file json)

### Json Packs
#### Persistent Data
Persistent Data that can be used to store some state related to packs which is specific to the user.
1. which packs are enabled for loading
2. which categories are enabled for rendering markers/trails
3. which markers have been activated by triggers previously and their wakeup timestamp.

#### Folder Structure of a pack
1. pack.json // has markers/categories and all that
2. images/   // folder that contains images named by their hashes
3. tbins/    // folder that contains trail binaries named by their hashes.

