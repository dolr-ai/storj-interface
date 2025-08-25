# Upload master.m3u8 file
POST {{host}}/hls/duplicate?publisher_user_id={{publisher}}&video_id={{video_id}}&is_nsfw=false&hls_file_name=master.m3u8&metadata[test]=value
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-hls-files/master.m3u8;
HTTP 200

# Upload playlist_0.m3u8 file
POST {{host}}/hls/duplicate?publisher_user_id={{publisher}}&video_id={{video_id}}&is_nsfw=false&hls_file_name=playlist_0.m3u8&metadata[test]=value
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-hls-files/playlist_0.m3u8;
HTTP 200

# Upload playlist_1.m3u8 file
POST {{host}}/hls/duplicate?publisher_user_id={{publisher}}&video_id={{video_id}}&is_nsfw=false&hls_file_name=playlist_1.m3u8&metadata[test]=value
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-hls-files/playlist_1.m3u8;
HTTP 200

# Upload playlist_2.m3u8 file
POST {{host}}/hls/duplicate?publisher_user_id={{publisher}}&video_id={{video_id}}&is_nsfw=false&hls_file_name=playlist_2.m3u8&metadata[test]=value
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-hls-files/playlist_2.m3u8;
HTTP 200

# Upload data_0_000.ts file
POST {{host}}/hls/duplicate?publisher_user_id={{publisher}}&video_id={{video_id}}&is_nsfw=false&hls_file_name=data_0_000.ts&metadata[test]=value
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-hls-files/data_0_000.ts;
HTTP 200

# Upload data_1_000.ts file
POST {{host}}/hls/duplicate?publisher_user_id={{publisher}}&video_id={{video_id}}&is_nsfw=false&hls_file_name=data_1_000.ts&metadata[test]=value
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-hls-files/data_1_000.ts;
HTTP 200

# Upload data_2_000.ts file
POST {{host}}/hls/duplicate?publisher_user_id={{publisher}}&video_id={{video_id}}&is_nsfw=false&hls_file_name=data_2_000.ts&metadata[test]=value
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-hls-files/data_2_000.ts;
HTTP 200

# Test with NSFW content
POST {{host}}/hls/duplicate?publisher_user_id={{publisher}}&video_id={{video_id}}_nsfw&is_nsfw=true&hls_file_name=master.m3u8&metadata[test]=value
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-hls-files/master.m3u8;
HTTP 200

# Test without auth token
POST {{host}}/hls/duplicate?publisher_user_id={{publisher}}&video_id={{video_id}}&is_nsfw=false&hls_file_name=master.m3u8&metadata[test]=value
Content-Type: application/octet-stream
file,test-hls-files/master.m3u8;
HTTP 401

# Test with wrong token
POST {{host}}/hls/duplicate?publisher_user_id={{publisher}}&video_id={{video_id}}&is_nsfw=false&hls_file_name=master.m3u8&metadata[test]=value
Authorization: Bearer wrongtoken
Content-Type: application/octet-stream
file,test-hls-files/master.m3u8;
HTTP 401

# Test missing required query parameters
POST {{host}}/hls/duplicate?video_id={{video_id}}&is_nsfw=false
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-hls-files/master.m3u8;
HTTP 400