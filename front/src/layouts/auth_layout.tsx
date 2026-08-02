import react from 'react';
import { Outlet } from 'react-router';
import Container from '@mui/material/Container';


export default class AuthLayout extends react.Component {
    render = () => {
        return (
            <>
                <header>
                </header>
                <main>
                    <Container>
                        <Outlet />
                    </Container>
                </main>
                <footer>
                </footer>
            </>
        );
    }
}


