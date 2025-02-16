curl -X POST http://localhost:5574/sync/sync \
    -H "Content-Type: application/json" \
    -d '{ "srcFs": "/home/dev/Documents/OBvault2/", "dstFs": "dge:OBvault", "createEmptySrcDirs": true,"_async": true }'


sync/copy: copy a directory from source remote to destination remote

This takes the following parameters:

    srcFs - a remote name string e.g. "drive:src" for the source
    dstFs - a remote name string e.g. "drive:dst" for the destination
    createEmptySrcDirs - create empty src directories on destination if set


curl -X POST http://localhost:5574/operations/copy \
    -H "Content-Type: application/json" \
    -d '{ 
          "srcFs": "/home/dev/Documents/OBvault2/", 
          "dstFs": "dge:/backup/", 
          "_async": true
        }'

curl -X POST http://localhost:5574/mount/mount \
    -H "Content-Type: application/json" \
    -d '{ "fs": "dge:", "mountPoint": "/home/dev/Documents/cloud/dge", "_async": true }'

 curl -X POST "http://localhost:5574/job/status" \
     -H "Content-Type: application/json" \
     -d '{ "jobid": "1" }'
{

curl -X POST "http://localhost:5574/job/status" \
     -H "Content-Type: application/json" \
     -d '{ "jobid": 17 }'
{
        "duration": 1.248352007,
        "endTime": "2025-02-15T19:36:43.64481602Z",
        "error": "",
        "finished": true,
        "group": "job/17",
        "id": 17,
        "output": {},
        "startTime": "2025-02-15T19:36:42.396464063Z",
        "success": true
}
