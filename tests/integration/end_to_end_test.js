const request = require('supertest');
const app = require('../../sovereign-chain/sovereign/app/app'); // Adjust the path as necessary

describe('End-to-End Tests for Sovereign Chain', () => {
    it('should start the blockchain and return status', async () => {
        const response = await request(app).get('/status');
        expect(response.status).toBe(200);
        expect(response.body).toHaveProperty('chain_id');
        expect(response.body).toHaveProperty('gas_fee', 0);
    });

    it('should add a block and return the correct data', async () => {
        const blockData = { data: 'Test transaction' };
        const response = await request(app).post('/write').send(blockData);
        expect(response.status).toBe(200);
        expect(response.body).toHaveProperty('index');
        expect(response.body).toHaveProperty('data', blockData.data);
    });

    it('should retrieve all blocks', async () => {
        const response = await request(app).get('/blocks');
        expect(response.status).toBe(200);
        expect(Array.isArray(response.body)).toBe(true);
    });
});