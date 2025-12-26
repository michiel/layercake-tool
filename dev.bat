@echo off
REM Layercake Development Script for Windows
REM Runs both frontend and backend in development mode

setlocal EnableDelayedExpansion

REM Configuration
set BACKEND_PORT=8080
set FRONTEND_PORT=1420
set BACKEND_DIR=.
set FRONTEND_DIR=frontend
if "%LOG_LEVEL%"=="" set LOG_LEVEL=info

echo [DEV] Starting Layercake development environment...
echo [DEV] Backend port: %BACKEND_PORT%
echo [DEV] Frontend port: %FRONTEND_PORT%
echo [DEV] Log level: %LOG_LEVEL%

REM Check if required directories exist
if not exist "%BACKEND_DIR%" (
    echo [ERROR] Backend directory '%BACKEND_DIR%' not found
    exit /b 1
)

if not exist "%FRONTEND_DIR%" (
    echo [ERROR] Frontend directory '%FRONTEND_DIR%' not found
    exit /b 1
)

REM Kill any existing processes on our ports (Windows)
for /f "tokens=5" %%a in ('netstat -aon ^| find ":%BACKEND_PORT%" ^| find "LISTENING"') do taskkill /f /pid %%a 2>nul
for /f "tokens=5" %%a in ('netstat -aon ^| find ":%FRONTEND_PORT%" ^| find "LISTENING"') do taskkill /f /pid %%a 2>nul

REM Initialize database if it doesn't exist
if not exist "layercake.db" (
    echo [DEV] Initializing database...
    cd "%BACKEND_DIR%"
    cargo run --bin layercake -- db init
    cd ..
    echo [SUCCESS] Database initialized
)

REM Start backend server
echo [DEV] Starting backend server...
cd "%BACKEND_DIR%"
start /b cmd /c "cargo run --bin layercake -- serve --port %BACKEND_PORT% --log-level %LOG_LEVEL% --cors-origin http://localhost:%FRONTEND_PORT% > backend.log 2>&1"
cd ..

REM Wait for backend to start
timeout /t 3 /nobreak >nul

echo [SUCCESS] Backend server started

REM Start frontend server
echo [DEV] Starting frontend server...
cd "%FRONTEND_DIR%"

REM Check if node_modules exists, install if not
if not exist "node_modules" (
    echo [DEV] Installing frontend dependencies...
    call npm install
)

REM Update environment file for backend connection
echo VITE_API_BASE_URL=http://localhost:%BACKEND_PORT% > .env.development.local

start /b cmd /c "npm run dev > ../frontend.log 2>&1"
cd ..

REM Wait for frontend to start
timeout /t 3 /nobreak >nul

echo [SUCCESS] Frontend server started

REM Display connection info
echo.
echo [SUCCESS] 🚀 Layercake development environment is ready!
echo.
echo 📊 Backend API:     http://localhost:%BACKEND_PORT%
echo 🌐 Frontend App:    http://localhost:%FRONTEND_PORT%
echo 📚 API Docs:        http://localhost:%BACKEND_PORT%/swagger-ui/
echo 🔍 GraphQL:         http://localhost:%BACKEND_PORT%/graphql
echo.
echo 📝 Logs:
echo    Backend: type backend.log
echo    Frontend: type frontend.log
echo.
echo Press Ctrl+C to stop all servers
echo.

REM Keep script running
:loop
timeout /t 30 /nobreak >nul
echo [DEV] Services still running...
goto loop
