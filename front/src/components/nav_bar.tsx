import react from 'react';
import AppBar from '@mui/material/AppBar';
import Box from '@mui/material/Box';
import Toolbar from '@mui/material/Toolbar';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import Menu from '@mui/material/Menu';
import MenuIcon from '@mui/icons-material/Menu';
import Container from '@mui/material/Container';
import Avatar from '@mui/material/Avatar';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import MenuItem from '@mui/material/MenuItem';
import AdbIcon from '@mui/icons-material/Adb';
import logo from '../assets/logo.svg';
import { NavLink } from "react-router";

const pages = [
    {
        "name": "Podcasts",
        "navigateTo": "/podcasts",
    },
    {
        "name": "Configuración",
        "navigateTo": "/configuration",
    },
    {
        "name": "Acerca de",
        "navigateTo": "/about",
    }];
const settings = [
    {
        "name": "Logout",
        "navigateTo": "/logout",
    }
];


export default class NavBar extends react.Component {
    state = {
        anchorElNav: null,
        anchorElUser: null,
    }
    handleOpenNavMenu = (event: React.MouseEvent<HTMLElement>) => {
        this.setState({ anchorElNav: event.currentTarget });
    }
    handleOpenUserMenu = (event: React.MouseEvent<HTMLElement>) => {
        this.setState({ anchorElUser: event.currentTarget });
    }

    handleCloseNavMenu = () => {
        this.setState({ anchorElNav: null });
    }

    handleCloseUserMenu = () => {
        this.setState({ anchorElUser: null });
    }

    render = () => {
        return (
            <>
                <AppBar position="static" sx={{ marginBottom: 2 }}>
                    <Container maxWidth="xl">
                        <Toolbar disableGutters>
                            <Avatar
                                alt="Podmixer"
                                src={logo}
                                sx={{ width: 32, height: 32, mx: "auto" }} />
                            <Typography
                                variant="h6"
                                noWrap
                                component="a"
                                href="/"
                                sx={{
                                    m: 1,
                                    mr: 2,
                                    display: { xs: 'none', md: 'flex' },
                                    fontWeight: 700,
                                    letterSpacing: '.3rem',
                                    color: 'inherit',
                                    textDecoration: 'none',
                                }}
                            >
                                Podmixer
                            </Typography>

                            <Box sx={{ flexGrow: 1, display: { xs: 'flex', md: 'none' } }}>
                                <IconButton
                                    size="large"
                                    aria-label="account of current user"
                                    aria-controls="menu-appbar"
                                    aria-haspopup="true"
                                    onClick={this.handleOpenNavMenu}
                                    color="inherit"
                                >
                                    <MenuIcon />
                                </IconButton>
                                <Menu
                                    id="menu-appbar"
                                    anchorEl={this.state.anchorElNav}
                                    anchorOrigin={{
                                        vertical: 'bottom',
                                        horizontal: 'left',
                                    }}
                                    keepMounted
                                    transformOrigin={{
                                        vertical: 'top',
                                        horizontal: 'left',
                                    }}
                                    open={Boolean(this.state.anchorElNav)}
                                    onClose={this.handleCloseNavMenu}
                                    sx={{ display: { xs: 'block', md: 'none' } }}
                                >
                                    {pages.map((page) => (
                                        <MenuItem key={page.name} onClick={this.handleCloseNavMenu}>
                                            <NavLink to={page.navigateTo} end>
                                                <Typography sx={{ textAlign: 'center' }}>{page.name}</Typography>
                                            </NavLink>
                                        </MenuItem>
                                    ))}
                                </Menu>
                            </Box>
                            <AdbIcon sx={{ display: { xs: 'flex', md: 'none' }, mr: 1 }} />
                            <Typography
                                variant="h5"
                                noWrap
                                component="a"
                                href="#app-bar-with-responsive-menu"
                                sx={{
                                    mr: 2,
                                    display: { xs: 'flex', md: 'none' },
                                    flexGrow: 1,
                                    fontFamily: 'monospace',
                                    fontWeight: 700,
                                    letterSpacing: '.3rem',
                                    color: 'inherit',
                                    textDecoration: 'none',
                                }}
                            >
                                LOGO
                            </Typography>
                            <Box sx={{ flexGrow: 1, display: { xs: 'none', md: 'flex' } }}>
                                {pages.map((page) => (
                                    <Button
                                        key={page.name}
                                        onClick={this.handleCloseNavMenu}
                                        sx={{ my: 2, color: 'white', display: 'block' }}
                                    >
                                        <NavLink to={page.navigateTo} end style={{textDecoration: 'none'}}>
                                            <Typography sx={{ textAlign: 'center' }}>{page.name}</Typography>
                                        </NavLink>
                                    </Button>
                                ))}
                            </Box>
                            <Box sx={{ flexGrow: 0 }}>
                                <Tooltip title="Open settings">
                                    <IconButton onClick={this.handleOpenUserMenu} sx={{ p: 0 }}>
                                        <Avatar alt="Remy Sharp" src="/static/images/avatar/2.jpg" />
                                    </IconButton>
                                </Tooltip>
                                <Menu
                                    sx={{ mt: '45px' }}
                                    id="menu-appbar"
                                    anchorEl={this.state.anchorElUser}
                                    anchorOrigin={{
                                        vertical: 'top',
                                        horizontal: 'right',
                                    }}
                                    keepMounted
                                    transformOrigin={{
                                        vertical: 'top',
                                        horizontal: 'right',
                                    }}
                                    open={Boolean(this.state.anchorElUser)}
                                    onClose={this.handleCloseUserMenu}
                                >
                                    {settings.map((setting) => (
                                        <MenuItem
                                            key={setting.name}
                                            onClick={
                                                () => {
                                                    this.handleCloseUserMenu();
                                                }
                                            }>
                                            <NavLink to={setting.navigateTo} end>
                                                <Typography sx={{ textAlign: 'center' }}>{setting.name}</Typography>
                                            </NavLink>
                                        </MenuItem>
                                    ))}
                                </Menu>
                            </Box>
                        </Toolbar>
                    </Container>
                </AppBar>
            </>
        );
    }
}



