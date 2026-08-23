// tests/fixtures/adversarial/typescript/nestjs_controller.ts

import { Controller, Get, Post, Put, Delete, Body, Param, Injectable } from '@nestjs/common';

// ⚠️ This looks dead but is a NestJS controller
@Controller('api/items')
export class ItemsController {
    constructor(private readonly itemsService: ItemsService) {}

    // ⚠️ Looks dead but is a GET endpoint
    @Get()
    findAll(): Item[] {
        return this.itemsService.findAll();
    }

    // ⚠️ Looks dead but is a GET endpoint with param
    @Get(':id')
    findOne(@Param('id') id: string): Item {
        return this.itemsService.findOne(id);
    }

    // ⚠️ Looks dead but is a POST endpoint
    @Post()
    create(@Body() item: Item): Item {
        return this.itemsService.create(item);
    }

    // ⚠️ Looks dead but is a PUT endpoint
    @Put(':id')
    update(@Param('id') id: string, @Body() item: Item): Item {
        return this.itemsService.update(id, item);
    }

    // ⚠️ Looks dead but is a DELETE endpoint
    @Delete(':id')
    delete(@Param('id') id: string): void {
        this.itemsService.delete(id);
    }
}

// ⚠️ This looks dead but is used by the controller
@Injectable()
export class ItemsService {
    private items: Item[] = [];

    findAll(): Item[] {
        return this.items;
    }

    findOne(id: string): Item {
        return this.items.find(i => i.id === id);
    }

    create(item: Item): Item {
        this.items.push(item);
        return item;
    }

    update(id: string, item: Item): Item {
        const index = this.items.findIndex(i => i.id === id);
        if (index !== -1) {
            this.items[index] = item;
        }
        return item;
    }

    delete(id: string): void {
        this.items = this.items.filter(i => i.id !== id);
    }
}

// Internal helper - should be considered dead
function internalHelper(): string {
    return "This is dead";
}

interface Item {
    id: string;
    name: string;
}
