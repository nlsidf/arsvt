const MAP_WIDTH = 50;
const MAP_HEIGHT = 50;

class Vec2 {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }
    
    add(v) {
        return new Vec2(this.x + v.x, this.y + v.y);
    }
    
    sub(v) {
        return new Vec2(this.x - v.x, this.y - v.y);
    }
    
    mul(s) {
        return new Vec2(this.x * s, this.y * s);
    }
    
    normalize() {
        const len = Math.sqrt(this.x * this.x + this.y * this.y);
        return len > 0 ? new Vec2(this.x / len, this.y / len) : new Vec2(0, 0);
    }
    
    rotate(angle) {
        const cos = Math.cos(angle);
        const sin = Math.sin(angle);
        return new Vec2(
            this.x * cos - this.y * sin,
            this.x * sin + this.y * cos
        );
    }
}

class MazeGenerator {
    constructor() {
        this.map = Array(MAP_WIDTH).fill(null).map(() => Array(MAP_HEIGHT).fill(1));
        this.visited = Array(MAP_WIDTH).fill(null).map(() => Array(MAP_HEIGHT).fill(false));
        this.startPos = { x: 0, y: 0 };
    }
    
    generate() {
        const startX = 1;
        const startY = 1;
        this.startPos = { x: startX + 0.5, y: startY + 0.5 };
        this.carvePath(startX, startY);
        
        for (let i = 0; i < 5; i++) {
            const x = Math.floor(Math.random() * (MAP_WIDTH - 2)) + 1;
            const y = Math.floor(Math.random() * (MAP_HEIGHT - 2)) + 1;
            if (this.map[x][y] === 0) {
                const wallType = Math.floor(Math.random() * 4) + 2;
                this.map[x][y] = wallType;
            }
        }
        
        return this.map;
    }
    
    carvePath(x, y) {
        this.visited[x][y] = true;
        this.map[x][y] = 0;
        
        const directions = [
            [0, -2], [2, 0], [0, 2], [-2, 0]
        ];
        
        for (let i = directions.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1));
            [directions[i], directions[j]] = [directions[j], directions[i]];
        }
        
        for (const [dx, dy] of directions) {
            const nx = x + dx;
            const ny = y + dy;
            
            if (nx > 0 && nx < MAP_WIDTH - 1 && ny > 0 && ny < MAP_HEIGHT - 1 && !this.visited[nx][ny]) {
                this.map[x + dx / 2][y + dy / 2] = 0;
                this.carvePath(nx, ny);
            }
        }
    }
    
    getStartPosition() {
        return this.startPos;
    }
}

class Camera {
    constructor(position, direction) {
        this.position = position;
        this.direction = direction.normalize();
        this.plane = new Vec2(0, 0.66);
        this.moveSpeed = 0.15;
        this.rotSpeed = 0.08;
        this.pitch = 0;
        this.bobPhase = 0;
    }
    
    moveForward(world) {
        const newPos = this.position.add(this.direction.mul(this.moveSpeed));
        if (!world.isWall(Math.floor(newPos.x), Math.floor(this.position.y))) {
            this.position.x = newPos.x;
        }
        if (!world.isWall(Math.floor(this.position.x), Math.floor(newPos.y))) {
            this.position.y = newPos.y;
        }
        this.bobPhase += 0.2;
    }
    
    moveBackward(world) {
        const newPos = this.position.sub(this.direction.mul(this.moveSpeed));
        if (!world.isWall(Math.floor(newPos.x), Math.floor(this.position.y))) {
            this.position.x = newPos.x;
        }
        if (!world.isWall(Math.floor(this.position.x), Math.floor(newPos.y))) {
            this.position.y = newPos.y;
        }
        this.bobPhase += 0.2;
    }
    
    strafeLeft(world) {
        const right = new Vec2(this.direction.y, -this.direction.x);
        const newPos = this.position.sub(right.mul(this.moveSpeed));
        if (!world.isWall(Math.floor(newPos.x), Math.floor(this.position.y))) {
            this.position.x = newPos.x;
        }
        if (!world.isWall(Math.floor(this.position.x), Math.floor(newPos.y))) {
            this.position.y = newPos.y;
        }
        this.bobPhase += 0.2;
    }
    
    strafeRight(world) {
        const right = new Vec2(this.direction.y, -this.direction.x);
        const newPos = this.position.add(right.mul(this.moveSpeed));
        if (!world.isWall(Math.floor(newPos.x), Math.floor(this.position.y))) {
            this.position.x = newPos.x;
        }
        if (!world.isWall(Math.floor(this.position.x), Math.floor(newPos.y))) {
            this.position.y = newPos.y;
        }
        this.bobPhase += 0.2;
    }
    
    rotate(angle) {
        const rotAngle = angle * this.rotSpeed;
        this.direction = this.direction.rotate(rotAngle);
        this.plane = this.plane.rotate(rotAngle);
    }
    
    getHorizonOffset() {
        const baseOffset = this.pitch * 150;
        const bobOffset = Math.sin(this.bobPhase) * 0.08 * 20;
        return Math.floor(baseOffset + bobOffset);
    }
}

class World {
    constructor() {
        this.regenerate();
    }
    
    regenerate() {
        const generator = new MazeGenerator();
        this.map = generator.generate();
        this.startPos = generator.getStartPosition();
        this.width = MAP_WIDTH;
        this.height = MAP_HEIGHT;
    }
    
    get(x, y) {
        if (x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT) {
            return 1;
        }
        return this.map[x][y];
    }
    
    isWall(x, y) {
        return this.get(x, y) !== 0;
    }
}

class Item {
    constructor(x, y, type) {
        this.x = x;
        this.y = y;
        this.type = type;
        this.collected = false;
    }
}

class Game {
    constructor() {
        this.canvas = document.getElementById('mainCanvas');
        this.ctx = this.canvas.getContext('2d');
        this.minimapCanvas = document.getElementById('minimap');
        this.minimapCtx = this.minimapCanvas.getContext('2d');
        
        this.resizeCanvas();
        window.addEventListener('resize', () => this.resizeCanvas());
        
        this.initTextures();
        
        this.world = new World();
        this.camera = new Camera(
            new Vec2(this.world.startPos.x, this.world.startPos.y),
            new Vec2(-1, 0)
        );
        
        this.items = [];
        this.generateItems();
        
        this.health = 100;
        this.steps = 0;
        this.coinsCollected = 0;
        this.keysCollected = 0;
        
        this.keys = {};
        this.running = true;
        this.lastTime = performance.now();
        this.fps = 60;
        
        this.setupControls();
        this.gameLoop();
    }
    
    initTextures() {
        this.skyCanvas = document.createElement('canvas');
        this.skyCanvas.width = 512;
        this.skyCanvas.height = 256;
        const skyCtx = this.skyCanvas.getContext('2d');
        
        const gradient = skyCtx.createLinearGradient(0, 0, 0, 256);
        gradient.addColorStop(0, '#1a4d7a');
        gradient.addColorStop(0.5, '#3b7fb8');
        gradient.addColorStop(1, '#87ceeb');
        skyCtx.fillStyle = gradient;
        skyCtx.fillRect(0, 0, 512, 256);
        
        for (let i = 0; i < 80; i++) {
            const x = Math.random() * 512;
            const y = Math.random() * 128;
            const size = Math.random() * 1.5;
            skyCtx.fillStyle = `rgba(255, 255, 255, ${0.4 + Math.random() * 0.6})`;
            skyCtx.fillRect(x, y, size, size);
        }
        
        for (let i = 0; i < 5; i++) {
            const x = Math.random() * 512;
            const y = 150 + Math.random() * 80;
            skyCtx.fillStyle = 'rgba(255, 255, 255, 0.35)';
            skyCtx.beginPath();
            skyCtx.ellipse(x, y, 35, 12, 0, 0, Math.PI * 2);
            skyCtx.fill();
        }
    }
    
    resizeCanvas() {
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
        
        const minimapSize = Math.min(200, Math.max(120, Math.floor(window.innerWidth * 0.12)));
        this.minimapCanvas.width = minimapSize;
        this.minimapCanvas.height = minimapSize;
        this.minimapCanvas.style.width = minimapSize + 'px';
        this.minimapCanvas.style.height = minimapSize + 'px';
    }
    
    generateItems() {
        this.items = [];
        for (let i = 0; i < 8; i++) {
            while (true) {
                const x = Math.floor(Math.random() * (this.world.width - 10)) + 5 + 0.5;
                const y = Math.floor(Math.random() * (this.world.height - 10)) + 5 + 0.5;
                if (!this.world.isWall(Math.floor(x), Math.floor(y))) {
                    this.items.push(new Item(x, y, 'coin'));
                    break;
                }
            }
        }
        for (let i = 0; i < 2; i++) {
            while (true) {
                const x = Math.floor(Math.random() * (this.world.width - 10)) + 5 + 0.5;
                const y = Math.floor(Math.random() * (this.world.height - 10)) + 5 + 0.5;
                if (!this.world.isWall(Math.floor(x), Math.floor(y))) {
                    this.items.push(new Item(x, y, 'key'));
                    break;
                }
            }
        }
    }
    
    setupControls() {
        window.addEventListener('keydown', (e) => {
            this.keys[e.key.toLowerCase()] = true;
            if (e.key.toLowerCase() === 'r') this.reset();
            if (e.key.toLowerCase() === 'n') this.newMaze();
        });
        
        window.addEventListener('keyup', (e) => {
            this.keys[e.key.toLowerCase()] = false;
        });
        
        this.mouseDown = false;
        this.lastMouseX = 0;
        
        this.canvas.addEventListener('mousedown', (e) => {
            this.mouseDown = true;
            this.lastMouseX = e.clientX;
        });
        
        this.canvas.addEventListener('mouseup', () => {
            this.mouseDown = false;
        });
        
        this.canvas.addEventListener('mouseleave', () => {
            this.mouseDown = false;
        });
        
        this.canvas.addEventListener('mousemove', (e) => {
            if (this.mouseDown) {
                const deltaX = e.clientX - this.lastMouseX;
                this.camera.rotate(deltaX * 0.05);
                this.lastMouseX = e.clientX;
            }
        });
        
        this.canvas.addEventListener('touchstart', (e) => {
            if (e.touches.length === 1) {
                this.mouseDown = true;
                this.lastMouseX = e.touches[0].clientX;
            }
        });
        
        this.canvas.addEventListener('touchend', () => {
            this.mouseDown = false;
        });
        
        this.canvas.addEventListener('touchmove', (e) => {
            if (this.mouseDown && e.touches.length === 1) {
                const deltaX = e.touches[0].clientX - this.lastMouseX;
                this.camera.rotate(deltaX * 0.05);
                this.lastMouseX = e.touches[0].clientX;
            }
        });
        
        this.joystickActive = false;
        this.joystickCenter = { x: 0, y: 0 };
        this.joystickOffset = { x: 0, y: 0 };
        
        const joystick = document.getElementById('joystick');
        const joystickContainer = document.getElementById('joystickContainer');
        
        const startJoystick = (e) => {
            this.joystickActive = true;
            const rect = joystickContainer.getBoundingClientRect();
            this.joystickCenter = {
                x: rect.left + rect.width / 2,
                y: rect.top + rect.height / 2
            };
        };
        
        const moveJoystick = (clientX, clientY) => {
            if (!this.joystickActive) return;
            
            const dx = clientX - this.joystickCenter.x;
            const dy = clientY - this.joystickCenter.y;
            const distance = Math.sqrt(dx * dx + dy * dy);
            const maxDistance = 40;
            
            if (distance > maxDistance) {
                this.joystickOffset.x = (dx / distance) * maxDistance;
                this.joystickOffset.y = (dy / distance) * maxDistance;
            } else {
                this.joystickOffset.x = dx;
                this.joystickOffset.y = dy;
            }
            
            joystick.style.transform = `translate(calc(-50% + ${this.joystickOffset.x}px), calc(-50% + ${this.joystickOffset.y}px))`;
            
            const threshold = 10;
            this.keys['w'] = this.joystickOffset.y < -threshold;
            this.keys['s'] = this.joystickOffset.y > threshold;
            this.keys['a'] = this.joystickOffset.x < -threshold;
            this.keys['d'] = this.joystickOffset.x > threshold;
        };
        
        const endJoystick = () => {
            this.joystickActive = false;
            this.joystickOffset = { x: 0, y: 0 };
            joystick.style.transform = 'translate(-50%, -50%)';
            this.keys['w'] = false;
            this.keys['s'] = false;
            this.keys['a'] = false;
            this.keys['d'] = false;
        };
        
        joystick.addEventListener('mousedown', startJoystick);
        joystick.addEventListener('touchstart', (e) => {
            e.preventDefault();
            startJoystick(e);
        });
        
        window.addEventListener('mousemove', (e) => {
            if (this.joystickActive) {
                moveJoystick(e.clientX, e.clientY);
            }
        });
        
        window.addEventListener('touchmove', (e) => {
            if (this.joystickActive && e.touches.length > 0) {
                moveJoystick(e.touches[0].clientX, e.touches[0].clientY);
            }
        });
        
        window.addEventListener('mouseup', endJoystick);
        window.addEventListener('touchend', endJoystick);
        
        document.getElementById('btnReset').addEventListener('click', () => this.reset());
        document.getElementById('btnNewMaze').addEventListener('click', () => this.newMaze());
    }
    
    handleInput() {
        const prevPos = { x: this.camera.position.x, y: this.camera.position.y };
        
        if (this.keys['w'] || this.keys['arrowup']) this.camera.moveForward(this.world);
        if (this.keys['s'] || this.keys['arrowdown']) this.camera.moveBackward(this.world);
        if (this.keys['a']) this.camera.strafeLeft(this.world);
        if (this.keys['d']) this.camera.strafeRight(this.world);
        if (this.keys['arrowleft']) this.camera.rotate(-1);
        if (this.keys['arrowright']) this.camera.rotate(1);
        
        if (prevPos.x !== this.camera.position.x || prevPos.y !== this.camera.position.y) {
            this.steps++;
        }
        
        this.checkItemCollection();
    }
    
    checkItemCollection() {
        for (const item of this.items) {
            if (!item.collected) {
                const dx = item.x - this.camera.position.x;
                const dy = item.y - this.camera.position.y;
                const dist = Math.sqrt(dx * dx + dy * dy);
                
                if (dist < 0.5) {
                    item.collected = true;
                    if (item.type === 'coin') {
                        this.coinsCollected++;
                    } else if (item.type === 'key') {
                        this.keysCollected++;
                    }
                }
            }
        }
    }
    
    reset() {
        this.camera.position = new Vec2(this.world.startPos.x, this.world.startPos.y);
        this.camera.direction = new Vec2(-1, 0).normalize();
        this.camera.plane = new Vec2(0, 0.66);
        this.camera.pitch = 0;
    }
    
    newMaze() {
        this.world.regenerate();
        this.camera = new Camera(
            new Vec2(this.world.startPos.x, this.world.startPos.y),
            new Vec2(-1, 0)
        );
        this.generateItems();
        this.coinsCollected = 0;
        this.keysCollected = 0;
        this.steps = 0;
    }
    
    render() {
        const width = this.canvas.width;
        const height = this.canvas.height;
        
        const horizonOffset = this.camera.getHorizonOffset();
        const halfHeight = Math.floor(height / 2);
        
        if (this.skyCanvas) {
            const skyWidth = 512;
            const angle = Math.atan2(this.camera.direction.y, this.camera.direction.x);
            const skyOffset = Math.floor((angle / (Math.PI * 2)) * skyWidth);
            
            const step = Math.max(1, Math.floor(width / 512));
            
            for (let x = 0; x < width; x += step) {
                const sx = Math.floor(((x / width) * skyWidth + skyOffset) % skyWidth);
                const wrappedSx = (sx + skyWidth) % skyWidth;
                this.ctx.drawImage(this.skyCanvas, wrappedSx, 0, step, 256, x, horizonOffset, step, halfHeight);
            }
        }
        
        const floorGradient = this.ctx.createLinearGradient(0, halfHeight + horizonOffset, 0, height);
        floorGradient.addColorStop(0, '#3a3a3a');
        floorGradient.addColorStop(1, '#1a1a1a');
        this.ctx.fillStyle = floorGradient;
        this.ctx.fillRect(0, halfHeight + horizonOffset, width, halfHeight);
        
        for (let x = 0; x < width; x++) {
            const cameraX = 2 * x / width - 1;
            const rayDir = new Vec2(
                this.camera.direction.x + this.camera.plane.x * cameraX,
                this.camera.direction.y + this.camera.plane.y * cameraX
            );
            
            let mapX = Math.floor(this.camera.position.x);
            let mapY = Math.floor(this.camera.position.y);
            
            const deltaDistX = Math.abs(1 / rayDir.x);
            const deltaDistY = Math.abs(1 / rayDir.y);
            
            let stepX, stepY;
            let sideDistX, sideDistY;
            
            if (rayDir.x < 0) {
                stepX = -1;
                sideDistX = (this.camera.position.x - mapX) * deltaDistX;
            } else {
                stepX = 1;
                sideDistX = (mapX + 1.0 - this.camera.position.x) * deltaDistX;
            }
            
            if (rayDir.y < 0) {
                stepY = -1;
                sideDistY = (this.camera.position.y - mapY) * deltaDistY;
            } else {
                stepY = 1;
                sideDistY = (mapY + 1.0 - this.camera.position.y) * deltaDistY;
            }
            
            let hit = false;
            let side = 0;
            let wallType = 0;
            
            while (!hit && mapX >= 0 && mapX < MAP_WIDTH && mapY >= 0 && mapY < MAP_HEIGHT) {
                if (sideDistX < sideDistY) {
                    sideDistX += deltaDistX;
                    mapX += stepX;
                    side = 0;
                } else {
                    sideDistY += deltaDistY;
                    mapY += stepY;
                    side = 1;
                }
                
                wallType = this.world.get(mapX, mapY);
                if (wallType !== 0) hit = true;
            }
            
            if (!hit) continue;
            
            let perpWallDist;
            if (side === 0) {
                perpWallDist = (mapX - this.camera.position.x + (1 - stepX) / 2) / rayDir.x;
            } else {
                perpWallDist = (mapY - this.camera.position.y + (1 - stepY) / 2) / rayDir.y;
            }
            
            const lineHeight = Math.floor(height / perpWallDist);
            let drawStart = Math.floor(-lineHeight / 2 + height / 2 + horizonOffset);
            let drawEnd = Math.floor(lineHeight / 2 + height / 2 + horizonOffset);
            
            drawStart = Math.max(0, drawStart);
            drawEnd = Math.min(height - 1, drawEnd);
            
            let wallX;
            if (side === 0) {
                wallX = this.camera.position.y + perpWallDist * rayDir.y;
            } else {
                wallX = this.camera.position.x + perpWallDist * rayDir.x;
            }
            wallX -= Math.floor(wallX);
            
            const distanceFactor = Math.max(0.25, 1.0 - perpWallDist / 22);
            const sideFactor = side === 0 ? 1.0 : 0.8;
            
            const baseColor = 220;
            const brightness = Math.floor(baseColor * distanceFactor * sideFactor);
            
            this.ctx.fillStyle = `rgb(${brightness},${brightness},${brightness})`;
            this.ctx.fillRect(x, drawStart, 1, drawEnd - drawStart);
            
            const wallHeight = drawEnd - drawStart;
            if (wallHeight > 30 && perpWallDist < 10) {
                const brickHeight = 25;
                const brickWidth = 20;
                
                const texX = Math.floor(wallX * 100) % brickWidth;
                const rowOffset = Math.floor((wallX * 100) / brickWidth) % 2;
                
                this.ctx.fillStyle = `rgba(100,100,100,${0.12 * distanceFactor})`;
                this.ctx.font = '12px monospace';
                this.ctx.textAlign = 'center';
                this.ctx.textBaseline = 'middle';
                
                const startRow = Math.floor(drawStart / brickHeight);
                const endRow = Math.ceil(drawEnd / brickHeight);
                
                for (let row = startRow; row <= endRow; row++) {
                    const yBase = row * brickHeight;
                    const offsetY = (rowOffset === 1) ? brickHeight / 2 : 0;
                    const yPos = yBase + offsetY;
                    
                    if (yPos >= drawStart - 5 && yPos <= drawEnd + 5) {
                        if (Math.abs(yPos - (drawStart + (drawEnd - drawStart) / 2)) < brickHeight / 2) {
                            if (texX < 3) {
                                this.ctx.fillText('│', x, yPos);
                            } else if (texX > brickWidth - 3) {
                                this.ctx.fillText('│', x, yPos);
                            }
                        }
                    }
                    
                    const topEdge = yBase + offsetY - 2;
                    if (topEdge >= drawStart && topEdge <= drawEnd && texX % 4 === 0) {
                        this.ctx.fillStyle = `rgba(80,80,80,${0.15 * distanceFactor})`;
                        this.ctx.fillText('─', x, topEdge);
                        this.ctx.fillStyle = `rgba(100,100,100,${0.12 * distanceFactor})`;
                    }
                }
            }
        }
        
        this.renderSprites();
        this.renderMinimap();
        this.updateHUD();
    }
    
    renderSprites() {
        const sprites = this.items.filter(item => !item.collected).map(item => ({
            x: item.x,
            y: item.y,
            type: item.type
        }));
        
        sprites.sort((a, b) => {
            const distA = (a.x - this.camera.position.x) ** 2 + (a.y - this.camera.position.y) ** 2;
            const distB = (b.x - this.camera.position.x) ** 2 + (b.y - this.camera.position.y) ** 2;
            return distB - distA;
        });
        
        for (const sprite of sprites) {
            const spriteX = sprite.x - this.camera.position.x;
            const spriteY = sprite.y - this.camera.position.y;
            
            const dist = Math.sqrt(spriteX * spriteX + spriteY * spriteY);
            if (dist > 15) continue;
            
            const invDet = 1.0 / (this.camera.plane.x * this.camera.direction.y - 
                                   this.camera.direction.x * this.camera.plane.y);
            
            const transformX = invDet * (this.camera.direction.y * spriteX - 
                                         this.camera.direction.x * spriteY);
            const transformY = invDet * (-this.camera.plane.y * spriteX + 
                                         this.camera.plane.x * spriteY);
            
            if (transformY <= 0.1) continue;
            
            const spriteScreenX = Math.floor((this.canvas.width / 2) * 
                                             (1 + transformX / transformY));
            
            const spriteHeight = Math.abs(Math.floor(this.canvas.height / transformY)) * 0.5;
            
            if (spriteHeight < 5) continue;
            
            const drawStartY = Math.floor(this.canvas.height / 2 - spriteHeight / 2);
            
            this.ctx.save();
            this.ctx.font = `${spriteHeight}px monospace`;
            this.ctx.textAlign = 'center';
            this.ctx.textBaseline = 'middle';
            
            if (sprite.type === 'coin') {
                this.ctx.fillStyle = '#FFD700';
                this.ctx.fillText('◉', spriteScreenX, drawStartY + spriteHeight / 2);
            } else if (sprite.type === 'key') {
                this.ctx.fillStyle = '#FFD700';
                this.ctx.fillText('🔑', spriteScreenX, drawStartY + spriteHeight / 2);
            }
            
            this.ctx.restore();
        }
    }
    
    renderMinimap() {
        const ctx = this.minimapCtx;
        const size = this.minimapCanvas.width;
        const scale = size / Math.max(this.world.width, this.world.height);
        
        ctx.fillStyle = '#000';
        ctx.fillRect(0, 0, size, size);
        
        const viewRange = 20;
        const camX = Math.floor(this.camera.position.x);
        const camY = Math.floor(this.camera.position.y);
        
        const startX = Math.max(0, camX - viewRange);
        const endX = Math.min(this.world.width, camX + viewRange);
        const startY = Math.max(0, camY - viewRange);
        const endY = Math.min(this.world.height, camY + viewRange);
        
        for (let x = startX; x < endX; x++) {
            for (let y = startY; y < endY; y++) {
                const wallType = this.world.get(x, y);
                if (wallType !== 0) {
                    ctx.fillStyle = '#444';
                    ctx.fillRect(x * scale, y * scale, scale, scale);
                }
            }
        }
        
        for (const item of this.items) {
            if (!item.collected) {
                ctx.fillStyle = item.type === 'coin' ? '#FFD700' : '#00FFFF';
                ctx.fillRect(item.x * scale - 2, item.y * scale - 2, 4, 4);
            }
        }
        
        ctx.fillStyle = '#0F0';
        ctx.beginPath();
        ctx.arc(this.camera.position.x * scale, this.camera.position.y * scale, 3, 0, Math.PI * 2);
        ctx.fill();
        
        ctx.strokeStyle = '#0F0';
        ctx.beginPath();
        ctx.moveTo(this.camera.position.x * scale, this.camera.position.y * scale);
        ctx.lineTo(
            (this.camera.position.x + this.camera.direction.x * 2) * scale,
            (this.camera.position.y + this.camera.direction.y * 2) * scale
        );
        ctx.stroke();
    }
    
    updateHUD() {
        document.getElementById('fps').textContent = Math.round(this.fps);
        document.getElementById('health').textContent = Math.round(this.health);
        document.getElementById('coins').textContent = this.coinsCollected;
        document.getElementById('keys').textContent = this.keysCollected;
        document.getElementById('steps').textContent = this.steps;
    }
    
    gameLoop() {
        if (!this.running) return;
        
        const currentTime = performance.now();
        const deltaTime = (currentTime - this.lastTime) / 1000;
        this.lastTime = currentTime;
        this.fps = 1 / deltaTime;
        
        this.handleInput();
        this.render();
        
        requestAnimationFrame(() => this.gameLoop());
    }
}

window.addEventListener('load', () => {
    new Game();
});
