import { BASE_URL } from '../constants';
import ApiResponse from '../models/api_response';

export const loadData = async (endpoint: string) => {
    console.log("Loading data");
    try {
        const response = await fetch(`${BASE_URL}/api/v1/${endpoint}`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
            },
        });
        return await response.json();
    } catch (error) {
        console.error('Error:', error);
        const msg = `Error: ${error}`;
        return new ApiResponse(500, msg);
    }
}

