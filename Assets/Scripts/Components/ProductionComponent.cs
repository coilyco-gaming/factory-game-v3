namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using System.Diagnostics;
    using System.Linq;
    using Assets.Scripts.Core;
    using UnityEngine;

    [Serializable]
    public class ProductionComponentCore
    {
        // TODO: Make this a "stack size" variable that varies by item
        public static uint InputBufferMultiplier = 2; // hold enough input buffer for 2 crafts

        private static uint PowerUsage = 10;
        public string Product;
        public GameContent.Item ProductItem =>
            this?.gameContent?.Items?.GetValueOrDefault(this.Product);
        public uint Quantity;
        public Dictionary<string, uint> Requests = new();
        public Dictionary<string, uint> Intermediates = new();
        public uint currentCraftProgress = 0;

        public double PercentCraftProgress =>
            this.ProductItem.CraftTime != 0
                ? Math.Round(
                    (double)(this.currentCraftProgress / (double)this.ProductItem.CraftTime),
                    2
                )
                : 1;

        public string PrecentProgressStatus => $"{this.PercentCraftProgress * 100}%";

        public bool outputBufferOccupied = false;
        private GameContent gameContent;
        private ResourcesComponentCore resources;
        private BatteryComponentCore battery;

        public class ProductionQueueRequests
        {
            public GameContent.Item Item;
            public uint CraftProgress;
            public uint Quantity;
        }

        // !!! TODO IMPORTANT: MAKE SURE TO CALL THESE IN THE WORLD OBJECT
        public void SetReservedCapacity(ResourcesComponentCore resources)
        {
            resources.reservedCapacity = this.ProductItem.Ingredients.ToDictionary(
                pair => pair.Key,
                pair => pair.Value * this.ProductItem.StackSize
            );
            resources.reservedCapacity[this.Product] = 1;
        }

        // !!! TODO IMPORTANT: MAKE SURE TO CALL THESE IN THE WORLD OBJECT
        public void SetInserterResourceTypes(List<ResourceInserterComponentCore> inserters)
        {
            inserters.ForEach(inserter =>
            {
                inserter.resourceType = this.ProductItem.Ingredients.Keys.ToList()[
                    inserters.IndexOf(inserter)
                ];
            });
        }

        public ProductionComponentCore(GameContent gameContent, string product)
        {
            this.gameContent = gameContent;
            this.Product = product;

            if (!this.ProductItem.CreateFromNothing && this.ProductItem.Ingredients.Count == 0)
            {
                throw new GameControllerCore.MisconfigurationException(
                    @$"Production component requires a product with ingredients.
                    The product {this.Product} has no ingredients and is not a manifester."
                );
            }
        }

        public List<Dictionary<uint, string>> Tick(
            GameControllerCore gameController,
            WorldObjectCore worldObject
        )
        {
            using Activity activity = gameController.backref.ActivitySource.StartActivity(
                this.GetType().Name
            );
            activity.SetTag("WorldObjectType", worldObject.worldObjectType);
            activity.SetTag("tick", gameController.backref.TickCount);
            activity.SetParentId(gameController.backref.WorldObjectTickActivity.Id);

            // If we have already started a craft, continue it.
            if (this.currentCraftProgress > 0)
            {
                // Perform no crafting if the battery is empty.
                try
                {
                    worldObject.battery.Energy -= ProductionComponentCore.PowerUsage;
                    this.currentCraftProgress += 1;
                    if (this.currentCraftProgress >= this.ProductItem.CraftTime)
                    {
                        worldObject.resources.ForceCreateResources(this.ProductItem.Name, 1);
                        this.currentCraftProgress = 0;
                    }
                }
                catch (BatteryComponentCore.BatteryCapacityException)
                {
                    return new();
                }
            }

            // If we have already produced the desired quantity, return.
            if (
                worldObject.resources.resources.GetValueOrDefault(this.Product, 0u)
                >= this.ProductItem.StackSize
            )
            {
                return new()
                {
                    new() { { gameController.backref.TickCount, "product output full" } },
                };
            }

            // If we would not have enough space to store the output, return.
            if (
                this.ProductItem.Weight > worldObject.resources.RemainingWeightCapacity
                || this.ProductItem.Volume > worldObject.resources.RemainingVolumeCapacity
            )
            {
                return new()
                {
                    new() { { gameController.backref.TickCount, "no space for product" } },
                };
            }

            // Check if we can craft the desired product.
            bool canCraft = true;
            foreach (KeyValuePair<string, uint> ingredient in this.ProductItem.Ingredients)
            {
                if (
                    !worldObject.resources.resources.ContainsKey(ingredient.Key)
                    || worldObject.resources.resources[ingredient.Key] < ingredient.Value
                )
                {
                    canCraft = false;
                    break;
                }
            }

            // If we can craft the product, do so.
            if (canCraft)
            {
                // Perform no crafting if the battery is empty.
                try
                {
                    worldObject.battery.Energy -= ProductionComponentCore.PowerUsage;
                    foreach (KeyValuePair<string, uint> ingredient in this.ProductItem.Ingredients)
                    {
                        worldObject.resources.ConsumeResources(ingredient.Key, ingredient.Value);
                    }
                    if (this.ProductItem.CraftTime == 1)
                    {
                        worldObject.resources.ForceCreateResources(this.ProductItem.Name, 1);
                    }
                    else
                    {
                        this.currentCraftProgress += 1;
                    }
                }
                catch (BatteryComponentCore.BatteryCapacityException)
                {
                    return new();
                }
            }
            else
            {
                return new()
                {
                    new() { { gameController.backref.TickCount, "need ingredients for product" } },
                };
            }

            return new();
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Xunit;
    using Xunit.Abstractions;

    public class TestProductionGameContent : GameContent
    {
        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                { "air", new Item("air", stackSize: 100, createFromNothing: true) },
                { "wood", new Item("wood", stackSize: 100) },
                { "nails", new Item("nails", stackSize: 100) },
                {
                    "planks",
                    new Item(
                        "planks",
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint> { { "wood", 5 } }
                    )
                },
                {
                    "wall",
                    new Item(
                        "wall",
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint>
                        {
                            { "planks", 5 },
                            { "nails", 5 },
                        }
                    )
                },
                {
                    "house",
                    new Item(
                        "house",
                        volume: 1000,
                        ingredients: new Dictionary<string, uint> { { "wall", 4 } }
                    )
                },
            };
    }

    public class TestProductionGameContentWithIron : GameContent
    {
        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                { "wood", new Item("wood", stackSize: 100) },
                { "iron", new Item("iron", stackSize: 100) },
                {
                    "nails",
                    new Item(
                        "nails",
                        stackSize: 100,
                        ingredients: new Dictionary<string, uint> { { "iron", 1 } }
                    )
                },
                {
                    "frame",
                    new Item(
                        "frame",
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint> { { "iron", 16 } }
                    )
                },
                {
                    "planks",
                    new Item(
                        "planks",
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint> { { "wood", 5 }, { "nails", 5 } }
                    )
                },
                {
                    "wall",
                    new Item(
                        "wall",
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint>
                        {
                            { "planks", 4 },
                            { "nails", 4 },
                            { "frame", 1 },
                        }
                    )
                },
            };
    }

    public class TestProductionCraftTime : GameContent
    {
        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                { "wood", new Item("wood", stackSize: 100) },
                {
                    "planks",
                    new Item(
                        "planks",
                        stackSize: 10,
                        craftTime: 3,
                        ingredients: new Dictionary<string, uint> { { "wood", 5 } }
                    )
                },
            };
    }

    public class ProductionComponentTest
    {
        private ITestOutputHelper testOutput;

        public ProductionComponentTest(ITestOutputHelper testOutputHelper)
        {
            this.testOutput = testOutputHelper;
        }

        [Fact]
        public void TestSimpleProduction()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null);

            ResourcesComponentCore resources = new(
                new TestProductionGameContent(),
                weightCapacity: 500,
                volumeCapacity: 500
            );
            resources.CreateResources("wood", 5);
            worldObject.resources = resources;

            BatteryComponentCore battery = new(100, 100);
            worldObject.battery = battery;

            Assert.Equal(5u, worldObject.resources.resources["wood"]);
            Assert.Equal(0u, worldObject.resources.resources.GetValueOrDefault("planks", 0u));

            ProductionComponentCore production = new(new TestProductionGameContent(), "planks");
            production.Tick(gameController, worldObject);

            Assert.Equal(0u, worldObject.resources.resources["wood"]);
            Assert.Equal(1u, worldObject.resources.resources["planks"]);
        }

        [Fact]
        public void TestOutputFilled()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null);

            ResourcesComponentCore resources = new(
                new TestProductionGameContent(),
                weightCapacity: 500,
                volumeCapacity: 500
            );
            resources.CreateResources("wall", 4);
            worldObject.resources = resources;

            BatteryComponentCore battery = new(100, 100);
            worldObject.battery = battery;

            ProductionComponentCore production = new(new TestProductionGameContent(), "house");
            production.Tick(gameController, worldObject);
            Assert.Equal(0u, worldObject.resources.resources.GetValueOrDefault("house", 0u));
            Assert.Equal(4u, worldObject.resources.resources["wall"]);
        }

        [Fact]
        public void TestCraftMultiple()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null);

            ResourcesComponentCore resources = new(
                new TestProductionGameContent(),
                weightCapacity: 500,
                volumeCapacity: 500
            );
            resources.CreateResources("wood", 20);
            worldObject.resources = resources;

            BatteryComponentCore battery = new(100, 100);
            worldObject.battery = battery;

            ProductionComponentCore production = new(new TestProductionGameContent(), "planks");
            production.Tick(gameController, worldObject);
            production.Tick(gameController, worldObject);
            production.Tick(gameController, worldObject);

            Assert.Equal(5u, resources.resources["wood"]);
            Assert.Equal(3u, resources.resources["planks"]);
        }

        [Fact]
        public void TestWithCraftTime()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new WorldObjectCore(null);
            ResourcesComponentCore resources = new(
                new TestProductionCraftTime(),
                weightCapacity: 500,
                volumeCapacity: 500
            );
            resources.CreateResources("wood", 5);
            worldObject.resources = resources;

            BatteryComponentCore battery = new(100, 100);
            List<ResourceInserterComponentCore> inserters = new()
            { //
                new("wood", 1),
            };
            worldObject.battery = battery;

            ProductionComponentCore production = new(new TestProductionCraftTime(), "planks");
            worldObject.production = production;

            production.Tick(gameController, worldObject);
            Assert.Equal(90u, battery.Energy);
            Assert.Equal(0u, resources.resources["wood"]);
            Assert.Equal(0u, resources.resources.GetValueOrDefault("planks", 0u));
            Assert.Equal(1u, production.currentCraftProgress);
            Assert.Equal("33%", production.PrecentProgressStatus);

            production.Tick(gameController, worldObject);
            Assert.Equal(80u, battery.Energy);
            Assert.Equal(0u, resources.resources["wood"]);
            Assert.Equal(0u, resources.resources.GetValueOrDefault("planks", 0u));
            Assert.Equal(2u, production.currentCraftProgress);
            Assert.Equal("67%", production.PrecentProgressStatus);

            production.Tick(gameController, worldObject);
            Assert.Equal(70u, battery.Energy);
            Assert.Equal(0u, resources.resources["wood"]);
            Assert.Equal(1u, resources.resources["planks"]);
        }

        [Fact]
        public void TestManifests()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null)
            {
                resources = new(
                    new TestProductionGameContent(),
                    weightCapacity: 500,
                    volumeCapacity: 500
                ),
                battery = new BatteryComponentCore(100, 100),
            };

            ProductionComponentCore production = new(new TestProductionGameContent(), "air");

            production.Tick(gameController, worldObject);
            Assert.Equal(1u, worldObject.resources.resources.GetValueOrDefault("air", 0u));
        }
    }
}
