-- Your SQL goes here

CREATE TABLE `album`(
    `album_id` INTEGER PRIMARY KEY,
    `name` TEXT NOT NULL,
    `artist_id` INTEGER,
    `image_id` INTEGER,
    `isComplation` INTEGER NOT NULL,
    `nameForSearch` TEXT,
    FOREIGN KEY (`artist_id`) REFERENCES `artist`(`id`),
    FOREIGN KEY (`image_id`) REFERENCES `image`(`id`)
);

CREATE TABLE `artist`(
    `artist_id` INTEGER PRIMARY KEY,
    `name` TEXT NOT NULL,
    `nameForSearch` TEXT
);

CREATE TABLE `category`(
    `category_id` INTEGER PRIMARY KEY,
    `menuitem_id` INTEGER NOT NULL,
    `sequenceNo` INTEGER NOT NULL,
    `isVisible` INTEGER NOT NULL
);

CREATE TABLE `color`(
    `color_id` INTEGER PRIMARY KEY,
    `name` TEXT NOT NULL
);

CREATE TABLE `content`(
    `content_id` INTEGER PRIMARY KEY,
    `title` TEXT,
    `titleForSearch` TEXT,
    `subtitle` TEXT,
    `bpmx100` INTEGER,
    `length` INTEGER,
    `trackNo` INTEGER,
    `discNo` INTEGER,
    `artist_id_artist` INTEGER,
    `artist_id_remixer` INTEGER,
    `originalartist_id` INTEGER,
    `artist_id_composer` INTEGER,
    `artist_id_lyricist` INTEGER,
    `album_id` INTEGER,
    `genre_id` INTEGER,
    `label_id` INTEGER,
    `key_id` INTEGER,
    `color_id` INTEGER,
    `image_id` INTEGER,
    `djComment` TEXT,
    `rating` INTEGER,
    `releaseYear` INTEGER,
    `releaseDate` TEXT,
    `dateCreated` TEXT NOT NULL,
    `dateAdded` TEXT NOT NULL,
    `path` TEXT NOT NULL,
    `fileName` TEXT,
    `fileSize` INTEGER,
    `fileType` INTEGER,
    `bitrate` INTEGER,
    `bitDepth` INTEGER,
    `samplingRate` INTEGER,
    `isrc` TEXT,
    `isHotCueAutoLoadOn` INTEGER,
    `isKuvoDeliverStatusOn` INTEGER,
    `kuvoDeliveryComment` TEXT,
    `masterDbId` INTEGER,
    `masterContentId` INTEGER,
    `analysisDataFilePath` TEXT,
    `analysedBits` INTEGER,
    `contentLink` INTEGER,
    `hasModified` INTEGER,
    `cueUpdateCount` INTEGER,
    `analysisDataUpdateCount` INTEGER,
    `informationUpdateCount` INTEGER,
    FOREIGN KEY (`album_id`) REFERENCES `album`(`id`),
    FOREIGN KEY (`genre_id`) REFERENCES `genre`(`id`),
    FOREIGN KEY (`label_id`) REFERENCES `label`(`id`),
    FOREIGN KEY (`key_id`) REFERENCES `key`(`id`),
    FOREIGN KEY (`color_id`) REFERENCES `color`(`id`),
    FOREIGN KEY (`image_id`) REFERENCES `image`(`id`)
);

CREATE TABLE `cue`(
    `cue_id` INTEGER PRIMARY KEY,
    `content_id` INTEGER NOT NULL,
    `kind` INTEGER,
    `colorTableIndex` INTEGER,
    `cueComment` TEXT,
    `isActiveLoop` INTEGER,
    `beatLoopNumerator` INTEGER,
    `beatLoopDenominator` INTEGER,
    `inUsec` INTEGER,
    `outUsec` INTEGER,
    `in150FramePerSec` INTEGER,
    `out150FramePerSec` INTEGER,
    `inMpegFrameNumber` INTEGER,
    `outMpegFrameNumber` INTEGER,
    `inMpegAbs` INTEGER,
    `outMpegAbs` INTEGER,
    `inDecodingStartFramePosition` INTEGER,
    `outDecodingStartFramePosition` INTEGER,
    `inFileOffsetInBlock` INTEGER,
    `outFileOffsetInBlock` INTEGER,
    `inNumberOfSampleInBlock` INTEGER,
    `outNumberOfSampleInBlock` INTEGER
);

CREATE TABLE `genre`(
    `genre_id` INTEGER PRIMARY KEY,
    `name` TEXT NOT NULL
);

CREATE TABLE `history`(
    `history_id` INTEGER PRIMARY KEY,
    `sequenceNo` INTEGER NOT NULL,
    `name` TEXT NOT NULL,
    `attribute` INTEGER NOT NULL,
    `history_id_parent` INTEGER NOT NULL
);

CREATE TABLE `history_content`(
    `history_id` INTEGER NOT NULL,
    `content_id` INTEGER NOT NULL,
    `sequenceNo` INTEGER NOT NULL,
    PRIMARY KEY(`history_id`, `content_id`)
);

CREATE TABLE `hotCueBankList`(
    `hotCueBankList_id` INTEGER PRIMARY KEY,
    `sequenceNo` INTEGER NOT NULL,
    `name` TEXT,
    `image_id` INTEGER,
    `attribute` INTEGER NOT NULL,
    `hotCueBankList_id_parent` INTEGER
);

CREATE TABLE `hotCueBankList_cue`(
    `hotCueBankList_id` INTEGER NOT NULL,
    `cue_id` INTEGER NOT NULL,
    `sequenceNo` INTEGER NOT NULL,
    PRIMARY KEY(`hotCueBankList_id`, `cue_id`)
);

CREATE TABLE `image`(
    `image_id` INTEGER PRIMARY KEY,
    `path` TEXT NOT NULL
);

CREATE TABLE `key`(
    `key_id` INTEGER PRIMARY KEY,
    `name` TEXT NOT NULL
);

CREATE TABLE `label`(
    `label_id` INTEGER PRIMARY KEY,
    `name` TEXT NOT NULL
);

CREATE TABLE `menuItem`(
    `menuItem_id` INTEGER PRIMARY KEY,
    `kind` INTEGER NOT NULL,
    `name` TEXT NOT NULL
);

CREATE TABLE `myTag`(
    `myTag_id` INTEGER PRIMARY KEY,
    `sequenceNo` INTEGER NOT NULL,
    `name` TEXT NOT NULL,
    `attribute` INTEGER NOT NULL,
    `myTag_id_parent` INTEGER NOT NULL
);

CREATE TABLE `myTag_content`(
    `myTag_id` INTEGER NOT NULL,
    `content_id` INTEGER NOT NULL,
    PRIMARY KEY(`myTag_id`, `content_id`)
);

CREATE TABLE `playlist`(
    `playlist_id` INTEGER PRIMARY KEY,
    `sequenceNo` INTEGER NOT NULL,
    `name` TEXT NOT NULL,
    `image_id` INTEGER,
    `attribute` INTEGER NOT NULL,
    `playlist_id_parent` INTEGER NOT NULL
);

CREATE TABLE `playlist_content`(
	`playlist_id` INTEGER NOT NULL,
	`content_id` INTEGER NOT NULL,
	`sequenceNo` INTEGER NOT NULL,
	PRIMARY KEY(`playlist_id`, `content_id`)
);

CREATE TABLE `property`(
    `deviceName` TEXT NOT NULL,
    `dbVersion` INTEGER NOT NULL,
    `numberOfContents` INTEGER NOT NULL,
    `createdDate` TEXT NOT NULL,
    `backGroundColorType` INTEGER NOT NULL,
    `myTagMasterDBID` INTEGER NOT NULL
);

CREATE TABLE `recommendedLike`(
    `content_id_1` INTEGER NOT NULL,
    `content_id_2` INTEGER NOT NULL,
    `rating` INTEGER NOT NULL,
    `createdDate` TEXT NOT NULL,
    PRIMARY KEY(`content_id_1`, `content_id_2`)
);

CREATE TABLE `sort`(
	`sort_id` INTEGER PRIMARY KEY,
	`menuItem_id` INTEGER NOT NULL,
	`sequenceNo` INTEGER NOT NULL,
	`isVisible` INTEGER NOT NULL,
	`isSelectedAsSubColumn` INTEGER NOT NULL
);
