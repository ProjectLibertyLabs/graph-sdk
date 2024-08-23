const axios = require('axios');
const fs = require('fs');
const path = require('path');
const os = require('os');
const version = require("./package.json").customProperties.uploadedBinariesVersion

const platform = os.platform();
console.log(platform)

let fileName;
switch (platform) {
    case 'darwin':
        fileName = 'libdsnp_graph_sdk_node.dylib';
        break;
    case 'win32':
        fileName = 'dsnp_graph_sdk_node.dll';
        break;
    case 'linux':
        fileName = 'libdsnp_graph_sdk_node.so';
        break;
    default:
        fileName = 'libdsnp_graph_sdk_node.so';
}

// URL of the file to download
const fileUrl = 'https://github.com/ProjectLibertyLabs/graph-sdk/releases/download/' + version + '/' + fileName;

// Complete output file path
const outputFile = path.join(__dirname, 'dsnp_graph_sdk_node.node');

// Download the file using axios
axios({
    method: 'get',
    url: fileUrl,
    responseType: 'stream'
})
    .then(response => {
        response.data.pipe(fs.createWriteStream(outputFile));
        console.log('Download completed:', outputFile);
    })
    .catch(error => {
        console.error('Error downloading file:', error.message);
    });