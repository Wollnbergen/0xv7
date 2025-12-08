const http = require('http');

const server = http.createServer((req, res) => {
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Content-Type', 'application/json');
    
    if (req.url === '/status') {
        res.writeHead(200);
        res.end(JSON.stringify({
            chain: 'sultan-1',
            version: '1.0.0',
            block_height: 145820,
            gas_price: 0.00,
            tps: 1230992,
            validators: 21,
            apy: 13.33,
            status: 'operational',
            completion: '70%'
        }));
    } else {
        res.writeHead(404);
        res.end(JSON.stringify({ error: 'Not found' }));
    }
});

const PORT = 1317;
server.listen(PORT, () => {
    console.log(`Sultan Chain API running on port ${PORT}`);
});
