import * as https from 'node:https';

/**
 * Pass the data to send as `event.data`, and the request options as
 * `event.options`. For more information see the HTTPS module documentation
 * at https://nodejs.org/api/https.html.
 *
 * Will succeed with the response body.
 */
export const handler = (event, context, callback) => {
    const req = https.request(event.options, (res) => {
        let body = '';
        console.log('Status:', res.statusCode);
        console.log('Headers:', JSON.stringify(res.headers));
        res.setEncoding('utf8');
        res.on('data', (chunk) => body += chunk);
        res.on('end', () => {
            console.log('Successfully processed HTTPS response');
            // If we know it's JSON, parse it
            if (res.headers['content-type'] === 'application/json') {
                body = JSON.parse(body);
            }
            callback(null, body);
        });
    });
    req.on('error', callback);
    req.write(JSON.stringify(event.data));
    req.end();
};

'use strict';

exports.handler = (event, context, callback) => {
  const request = event.Records[0].cf.request;
  const headers = request.headers;

  // Check if Authorization header exists
  if (headers.authorization && headers.authorization.length > 0) {
    const authorizationHeader = headers.authorization[0].value;

    // Log for debugging (optional)
    console.log('Authorization Header found:', authorizationHeader);

    //  Optionally, perform JWT validation here before forwarding

    // Modify the request by forwarding the Authorization header
      request.headers['authorization'] = [{key: 'Authorization', value: authorizationHeader}];
  } else {
      console.log('No Authorization header found');
  }

  // Return the modified request
  callback(null, request);
};