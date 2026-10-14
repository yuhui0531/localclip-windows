CREATE TABLE clips(id TEXT PRIMARY KEY,contentHash TEXT NOT NULL UNIQUE,
 firstCopiedAtMs INTEGER NOT NULL,lastCopiedAtMs INTEGER NOT NULL,
 copyCount INTEGER NOT NULL CHECK(copyCount>=1),kind TEXT NOT NULL,
 isFavorite INTEGER NOT NULL CHECK(isFavorite IN(0,1)),sourceAppID TEXT,
 sourceName TEXT NOT NULL,sourceBasis TEXT NOT NULL,previewText TEXT NOT NULL,
 title TEXT,normalizedTitle TEXT);
CREATE TABLE items(clipID TEXT NOT NULL REFERENCES clips(id) ON DELETE CASCADE,
 itemIndex INTEGER NOT NULL CHECK(itemIndex>=0),declaredTypes TEXT NOT NULL,PRIMARY KEY(clipID,itemIndex));
CREATE TABLE blobs(digest TEXT PRIMARY KEY,byteCount INTEGER NOT NULL CHECK(byteCount>=0),
 payload BLOB NOT NULL,CHECK(length(payload)=byteCount));
CREATE TABLE representations(clipID TEXT NOT NULL,itemIndex INTEGER NOT NULL,type TEXT NOT NULL,
 blobDigest TEXT NOT NULL REFERENCES blobs(digest),PRIMARY KEY(clipID,itemIndex,type),
 FOREIGN KEY(clipID,itemIndex) REFERENCES items(clipID,itemIndex) ON DELETE CASCADE);
CREATE TABLE text_content(clipID TEXT NOT NULL,itemIndex INTEGER NOT NULL,plainText TEXT NOT NULL,
 normalizedText TEXT NOT NULL,PRIMARY KEY(clipID,itemIndex),
 FOREIGN KEY(clipID,itemIndex) REFERENCES items(clipID,itemIndex) ON DELETE CASCADE);
CREATE TABLE thumbnails(clipID TEXT PRIMARY KEY REFERENCES clips(id) ON DELETE CASCADE,
 revision INTEGER NOT NULL,encodedImage BLOB NOT NULL,byteCount INTEGER NOT NULL,CHECK(length(encodedImage)=byteCount));
CREATE TABLE settings(singletonID INTEGER PRIMARY KEY CHECK(singletonID=1),schemaVersion INTEGER NOT NULL,document TEXT NOT NULL);
CREATE TABLE store_meta(singletonID INTEGER PRIMARY KEY CHECK(singletonID=1),schemaVersion INTEGER NOT NULL,storeRevision INTEGER NOT NULL,lastOperationID TEXT NOT NULL);
INSERT INTO store_meta VALUES(1,1,0,'');
CREATE TABLE categories(id TEXT PRIMARY KEY,name TEXT NOT NULL,normalizedName TEXT NOT NULL UNIQUE,
 createdAtMs INTEGER NOT NULL,colorHex TEXT NOT NULL,position INTEGER NOT NULL);
CREATE TABLE clip_categories(categoryID TEXT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
 clipID TEXT NOT NULL REFERENCES clips(id) ON DELETE CASCADE,position INTEGER NOT NULL,PRIMARY KEY(categoryID,clipID));
CREATE INDEX recent ON clips(lastCopiedAtMs DESC,id);
CREATE INDEX favorites ON clips(isFavorite,lastCopiedAtMs,firstCopiedAtMs,id);
CREATE INDEX blob_references ON representations(blobDigest);
CREATE INDEX clip_category_memberships ON clip_categories(clipID,categoryID);
PRAGMA user_version=1;
