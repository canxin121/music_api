# Details

Date : 2024-06-30 19:27:37

Directory d:\\Git\\music_api

Total : 45 files,  6935 codes, 313 comments, 795 blanks, all 8043 lines

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [Cargo.lock](/Cargo.lock) | TOML | 2,308 | 2 | 263 | 2,573 |
| [Cargo.toml](/Cargo.toml) | TOML | 37 | 1 | 3 | 41 |
| [examples/online.rs](/examples/online.rs) | Rust | 33 | 10 | 4 | 47 |
| [examples/sql.rs](/examples/sql.rs) | Rust | 43 | 9 | 4 | 56 |
| [readme.md](/readme.md) | Markdown | 19 | 0 | 14 | 33 |
| [src/factory/mod.rs](/src/factory/mod.rs) | Rust | 2 | 0 | 1 | 3 |
| [src/factory/online_factory/aggregator_search.rs](/src/factory/online_factory/aggregator_search.rs) | Rust | 197 | 0 | 14 | 211 |
| [src/factory/online_factory/mod.rs](/src/factory/online_factory/mod.rs) | Rust | 4 | 0 | 3 | 7 |
| [src/factory/online_factory/musiclist_online.rs](/src/factory/online_factory/musiclist_online.rs) | Rust | 106 | 1 | 9 | 116 |
| [src/factory/sql_factory/db_action.rs](/src/factory/sql_factory/db_action.rs) | Rust | 126 | 18 | 18 | 162 |
| [src/factory/sql_factory/mod.rs](/src/factory/sql_factory/mod.rs) | Rust | 37 | 20 | 13 | 70 |
| [src/factory/sql_factory/music_action.rs](/src/factory/sql_factory/music_action.rs) | Rust | 508 | 33 | 61 | 602 |
| [src/factory/sql_factory/musicdata_action.rs](/src/factory/sql_factory/musicdata_action.rs) | Rust | 133 | 8 | 14 | 155 |
| [src/factory/sql_factory/musiclist_action.rs](/src/factory/sql_factory/musiclist_action.rs) | Rust | 272 | 18 | 24 | 314 |
| [src/factory/sql_factory/sql_musiclist.rs](/src/factory/sql_factory/sql_musiclist.rs) | Rust | 27 | 0 | 7 | 34 |
| [src/factory/sql_factory/util.rs](/src/factory/sql_factory/util.rs) | Rust | 0 | 0 | 2 | 2 |
| [src/filter.rs](/src/filter.rs) | Rust | 49 | 1 | 5 | 55 |
| [src/lib.rs](/src/lib.rs) | Rust | 15 | 1 | 2 | 18 |
| [src/music.rs](/src/music.rs) | Rust | 84 | 19 | 11 | 114 |
| [src/music_aggregator/mod.rs](/src/music_aggregator/mod.rs) | Rust | 76 | 20 | 18 | 114 |
| [src/music_aggregator/music_aggregator_online.rs](/src/music_aggregator/music_aggregator_online.rs) | Rust | 203 | 3 | 21 | 227 |
| [src/music_aggregator/music_aggregator_sql.rs](/src/music_aggregator/music_aggregator_sql.rs) | Rust | 336 | 16 | 30 | 382 |
| [src/music_list.rs](/src/music_list.rs) | Rust | 49 | 0 | 6 | 55 |
| [src/platform_integrator/kuwo/kuwo_album.rs](/src/platform_integrator/kuwo/kuwo_album.rs) | Rust | 134 | 0 | 19 | 153 |
| [src/platform_integrator/kuwo/kuwo_lyric.rs](/src/platform_integrator/kuwo/kuwo_lyric.rs) | Rust | 49 | 0 | 8 | 57 |
| [src/platform_integrator/kuwo/kuwo_music.rs](/src/platform_integrator/kuwo/kuwo_music.rs) | Rust | 307 | 3 | 31 | 341 |
| [src/platform_integrator/kuwo/kuwo_music_info.rs](/src/platform_integrator/kuwo/kuwo_music_info.rs) | Rust | 36 | 0 | 7 | 43 |
| [src/platform_integrator/kuwo/kuwo_music_list.rs](/src/platform_integrator/kuwo/kuwo_music_list.rs) | Rust | 172 | 0 | 23 | 195 |
| [src/platform_integrator/kuwo/kuwo_pic.rs](/src/platform_integrator/kuwo/kuwo_pic.rs) | Rust | 20 | 0 | 3 | 23 |
| [src/platform_integrator/kuwo/kuwo_quality.rs](/src/platform_integrator/kuwo/kuwo_quality.rs) | Rust | 59 | 0 | 5 | 64 |
| [src/platform_integrator/kuwo/kuwo_search.rs](/src/platform_integrator/kuwo/kuwo_search.rs) | Rust | 70 | 6 | 13 | 89 |
| [src/platform_integrator/kuwo/mod.rs](/src/platform_integrator/kuwo/mod.rs) | Rust | 14 | 0 | 2 | 16 |
| [src/platform_integrator/kuwo/util.rs](/src/platform_integrator/kuwo/util.rs) | Rust | 15 | 0 | 1 | 16 |
| [src/platform_integrator/mod.rs](/src/platform_integrator/mod.rs) | Rust | 5 | 0 | 2 | 7 |
| [src/platform_integrator/utils/mod.rs](/src/platform_integrator/utils/mod.rs) | Rust | 25 | 7 | 4 | 36 |
| [src/platform_integrator/wangyi/encrypt.rs](/src/platform_integrator/wangyi/encrypt.rs) | Rust | 164 | 10 | 26 | 200 |
| [src/platform_integrator/wangyi/mod.rs](/src/platform_integrator/wangyi/mod.rs) | Rust | 15 | 0 | 3 | 18 |
| [src/platform_integrator/wangyi/request.rs](/src/platform_integrator/wangyi/request.rs) | Rust | 12 | 0 | 3 | 15 |
| [src/platform_integrator/wangyi/wy_album.rs](/src/platform_integrator/wangyi/wy_album.rs) | Rust | 87 | 1 | 11 | 99 |
| [src/platform_integrator/wangyi/wy_lyric.rs](/src/platform_integrator/wangyi/wy_lyric.rs) | Rust | 84 | 2 | 6 | 92 |
| [src/platform_integrator/wangyi/wy_music.rs](/src/platform_integrator/wangyi/wy_music.rs) | Rust | 592 | 89 | 31 | 712 |
| [src/platform_integrator/wangyi/wy_music_detail.rs](/src/platform_integrator/wangyi/wy_music_detail.rs) | Rust | 56 | 1 | 7 | 64 |
| [src/platform_integrator/wangyi/wy_musiclist.rs](/src/platform_integrator/wangyi/wy_musiclist.rs) | Rust | 202 | 12 | 23 | 237 |
| [src/platform_integrator/wangyi/wy_search.rs](/src/platform_integrator/wangyi/wy_search.rs) | Rust | 77 | 2 | 13 | 92 |
| [src/util.rs](/src/util.rs) | Rust | 76 | 0 | 7 | 83 |

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)