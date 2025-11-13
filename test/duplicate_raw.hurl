# proper input - SFW video
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&video_id={{video_id}}_raw&is_nsfw=false
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 200

# Segregate nsfw videos
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&video_id={{video_id}}_raw_nsfw&is_nsfw=true
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 200

# Without auth token
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&video_id={{video_id}}_raw&is_nsfw=false
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 401

# With wrong token
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&video_id={{video_id}}_raw&is_nsfw=false
Authorization: Bearer wrongtoken
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 401

# missing required query parameter - publisher_user_id
POST {{host}}/duplicate_raw?video_id={{video_id}}_raw&is_nsfw=false
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 422

# missing required query parameter - video_id
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&is_nsfw=false
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 422

# missing required query parameter - is_nsfw
POST {{host}}/duplicate_raw?publisher_user_id={{publisher}}&video_id={{video_id}}_raw
Authorization: Bearer {{api_token}}
Content-Type: application/octet-stream
file,test-raw-files/test-raw-video.mp4;
HTTP 422
