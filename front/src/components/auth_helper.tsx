import { jwtDecode } from 'jwt-decode';

interface Decoded {
    exp: number;
}

export default class AuthHelperMethods {

    loggedIn = () => {
        const token = this.getToken();
        return !!token && !this.isTokenExpired(token);
    }

    isTokenExpired = (token: string) => {
        try {
            const decoded: Decoded = jwtDecode(token);
            if (decoded.exp < Date.now() / 1000) { // Checking if token is expired.
                return true;
            }
            else
                return false;
        }
        catch (err) {
            console.log("expired check failed!");
            return false;
        }
    }

    setToken = (token: string) => {
        localStorage.setItem('token', token);
    }

    getToken = () => {
        return localStorage.getItem('token');
    }

    logout = () => {
        localStorage.removeItem('token');
    }

    getConfirm = () => {
        const token = this.getToken();
        if(token){
            let answer = jwtDecode(token);
            console.log("Recieved answer!");
            return answer;
        }
        return null;
    }

}

