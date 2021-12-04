## Handling of Marker Packs

There's two types of Marker Packs:
1. Xml Packs (zipped with taco extension or non-zipped in a folder)
2. Json packs (single file json or a git repo)

### Json Packs
#### Persistent Data
Persistent Data that can be used to store some state related to packs which is specific to the user.
1. which packs are enabled for loading
2. which categories are enabled for loading
3. which markers have been activated by triggers previously and their reset times/dates

#### Pack file format (Git Repo)
1. must be a git repo with a release branch. the overlay will create a user branch where changes that user makes are saved. (like increasing the size of all markers on bigger screens)
2. pack.json in the root folder which has details about a markerpack like authors, id etc.. 
3. IF pack id is enabled for loading, we load cats.json in the root folder which must have a recursive object. this represents the selection menu ui tree, and also has cat ids along with the name.We now also load images.json and trl.json which contain the names/ids of images/trails with hashes as their keys. 
4. IF cat id is enabled, we check the cats folder for a subdirectory named with the cat uuid. under that directory, we get the cat.json which has cat description like name or authors, and we collect the names of all files that are numbers which are mapids where markers are to be shown.
5. everytime we load into a new map, we check all the enabled cats and only show the cats which have the current map id. then, we read those files to load them and display them. we display static and dynamic markers separately. for dynamic markers, we just loop through only refreshing the markers to be drawn if some marker changes. we will get the images/trails based on the previously loaded images/trl maps. there should be a images/ and trls/ subdirectories which contain the image/trl files with the name of the hash.


the above only works for READ ONLY Packs. the only file that we didn't cover is the tags.json in each cat directory which has a list of strings each of which refer to an array of UUIDs representing markers/trails which have that tag(string). this can be used to represent groups of markers without complicated stuff like subcategories or hidden categories. this will also make sure that we only care about tags IF we are editing a pack, otherwise, there's no reason to have it. 
##### Important Files
1. pack.json at pack root: pack description
2. cats.json at pack root: Cat selection tree with cat name and id
3. images.json at pack root: has the images metadata with their hashes
4. trls.json at pack root: has the trail data names/hashes
5. images/ folder at pack root: contains png images named by their rgba8 pixels hash.
6. trls/ folder at pack root: contains trl data files named by their nodes of Vec3 hash.
7. cats/ folder at pack root: contains sub directories named by the UUID of cats
8. cat.json at cat folder: contains cat description
9. map_id.json at cat folder: map_id is replaced by a mapid (u16). contains markers/trails 
10. tags.json at cat folder: contains tags(strings) which list the uuids of markers/trails that they associate with.

##### RWPack
Pack editing is a hard choice for performance. 
1. We keep the packs separate, clone the category when editing, and finally, when we save the changes, we assign the new cat back to the pack and write to the file on disk. but, we have to batch edited category markers separately and if we don't disable the category in markerpack, they will get rendered twice (from pack's original cat and edited cat). merits: simple and low render buffer updates. 
2. we go YOLO and just edit them live by keeping minimal state like the cat id being edited/marker id being edited etc.. but every time there's a change in marker, we must update the buffers. OTOH, we don't bother with cloning cats or duplicate rendering or having separate buffers for edited markers. 