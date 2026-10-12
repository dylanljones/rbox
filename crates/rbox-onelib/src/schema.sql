-- Copyright (C) 2026 Dylan Jones
-- SPDX-License-Identifier: GPL-3.0-only

-- rekordbox's OneLibrary schema

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

-- Default values in an empty database

INSERT INTO "category" VALUES (1,1,0,0);
INSERT INTO "category" VALUES (2,2,1,1);
INSERT INTO "category" VALUES (3,3,2,1);
INSERT INTO "category" VALUES (4,4,3,1);
INSERT INTO "category" VALUES (5,17,5,1);
INSERT INTO "category" VALUES (6,5,0,0);
INSERT INTO "category" VALUES (7,6,0,0);
INSERT INTO "category" VALUES (8,7,0,0);
INSERT INTO "category" VALUES (9,8,0,0);
INSERT INTO "category" VALUES (10,9,0,0);
INSERT INTO "category" VALUES (11,10,0,0);
INSERT INTO "category" VALUES (12,11,4,1);
INSERT INTO "category" VALUES (15,13,0,0);
INSERT INTO "category" VALUES (17,24,9,1);
INSERT INTO "category" VALUES (18,20,7,1);
INSERT INTO "category" VALUES (19,14,0,0);
INSERT INTO "category" VALUES (20,15,0,0);
INSERT INTO "category" VALUES (21,16,0,0);
INSERT INTO "category" VALUES (22,19,6,1);
INSERT INTO "category" VALUES (23,18,11,1);
INSERT INTO "category" VALUES (26,27,8,1);
INSERT INTO "category" VALUES (27,22,10,1);
INSERT INTO "color" VALUES (1,'Pink');
INSERT INTO "color" VALUES (2,'Red');
INSERT INTO "color" VALUES (3,'Orange');
INSERT INTO "color" VALUES (4,'Yellow');
INSERT INTO "color" VALUES (5,'Green');
INSERT INTO "color" VALUES (6,'Aqua');
INSERT INTO "color" VALUES (7,'Blue');
INSERT INTO "color" VALUES (8,'Purple');
INSERT INTO "menuItem" VALUES (1,128,'￺GENRE￻');
INSERT INTO "menuItem" VALUES (2,129,'￺ARTIST￻');
INSERT INTO "menuItem" VALUES (3,130,'￺ALBUM￻');
INSERT INTO "menuItem" VALUES (4,131,'￺TRACK￻');
INSERT INTO "menuItem" VALUES (5,133,'￺BPM￻');
INSERT INTO "menuItem" VALUES (6,134,'￺RATING￻');
INSERT INTO "menuItem" VALUES (7,135,'￺YEAR￻');
INSERT INTO "menuItem" VALUES (8,136,'￺REMIXER￻');
INSERT INTO "menuItem" VALUES (9,137,'￺LABEL￻');
INSERT INTO "menuItem" VALUES (10,138,'￺ORIGINAL ARTIST￻');
INSERT INTO "menuItem" VALUES (11,139,'￺KEY￻');
INSERT INTO "menuItem" VALUES (12,141,'￺CUE￻');
INSERT INTO "menuItem" VALUES (13,142,'￺COLOR￻');
INSERT INTO "menuItem" VALUES (14,146,'￺TIME￻');
INSERT INTO "menuItem" VALUES (15,147,'￺BITRATE￻');
INSERT INTO "menuItem" VALUES (16,148,'￺FILE NAME￻');
INSERT INTO "menuItem" VALUES (17,132,'￺PLAYLIST￻');
INSERT INTO "menuItem" VALUES (18,152,'￺HOT CUE BANK￻');
INSERT INTO "menuItem" VALUES (19,149,'￺HISTORY￻');
INSERT INTO "menuItem" VALUES (20,145,'￺SEARCH￻');
INSERT INTO "menuItem" VALUES (21,150,'￺COMMENTS￻');
INSERT INTO "menuItem" VALUES (22,140,'￺DATE ADDED￻');
INSERT INTO "menuItem" VALUES (23,151,'￺DJ PLAY COUNT￻');
INSERT INTO "menuItem" VALUES (24,144,'￺FOLDER￻');
INSERT INTO "menuItem" VALUES (25,161,'￺DEFAULT￻');
INSERT INTO "menuItem" VALUES (26,162,'￺ALPHABET￻');
INSERT INTO "menuItem" VALUES (27,170,'￺MATCHING￻');
INSERT INTO "sort" VALUES (0,25,1,1,0);
INSERT INTO "sort" VALUES (1,26,2,1,0);
INSERT INTO "sort" VALUES (2,2,3,1,0);
INSERT INTO "sort" VALUES (3,3,4,1,0);
INSERT INTO "sort" VALUES (4,5,5,1,0);
INSERT INTO "sort" VALUES (5,6,6,1,0);
INSERT INTO "sort" VALUES (6,1,0,0,0);
INSERT INTO "sort" VALUES (7,21,0,0,0);
INSERT INTO "sort" VALUES (8,14,0,0,0);
INSERT INTO "sort" VALUES (9,8,0,0,0);
INSERT INTO "sort" VALUES (10,9,0,0,0);
INSERT INTO "sort" VALUES (11,10,0,0,0);
INSERT INTO "sort" VALUES (12,11,7,1,0);
INSERT INTO "sort" VALUES (13,15,0,0,0);
INSERT INTO "sort" VALUES (15,13,0,0,0);
INSERT INTO "sort" VALUES (16,23,0,0,0);
INSERT INTO "sort" VALUES (17,22,0,0,0);
