import react from 'react';
import { Outlet } from 'react-router';
import Container from '@mui/material/Container';
import NavBar from '../components/nav_bar';


export default class MainLayout extends react.Component {

    render = () => {
        return (
            <>
                <header>
                    <NavBar />
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


