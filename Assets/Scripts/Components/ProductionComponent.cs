namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Core;

    public class ProductionComponentCore
    {
        // TODO: Make this a "stack size" variable that varies by item
        public static uint InputBufferMultiplier = 2; // hold enough input buffer for 2 crafts

        // TODO: production uses power on every step + every create
        public static uint PowerUsage = 10;
        public string Product;
        public GameContent.Item ProductItem => this.gameContent.Items[this.Product];
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
                : 0;

        public string PrecentProgressStatus => $"{this.PercentCraftProgress * 100}%";

        public bool outputBufferFull = false;
        private GameContent gameContent;
        private ResourcesComponentCore resources;
        private BatteryComponentCore battery;
        private List<InserterComponentCore> inserters;

        public class ProductionQueueRequests
        {
            public GameContent.Item Item;
            public uint CraftProgress;
            public uint Quantity;
        }

        public ProductionComponentCore(
            GameContent gameContent,
            ResourcesComponentCore resources,
            BatteryComponentCore battery,
            List<InserterComponentCore> inserters,
            string product
        )
        {
            this.gameContent = gameContent;
            this.Product = product;

            this.battery =
                battery
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Production component requires a battery component"
                );

            this.resources =
                resources
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Production component requires a resource component"
                );
            this.resources.reservedCapacity = this.ProductItem.Ingredients.ToDictionary(
                pair => pair.Key,
                pair => pair.Value * ProductionComponentCore.InputBufferMultiplier
            );
            this.resources.reservedCapacity[this.Product] = 1;

            if (inserters == null || inserters.Count != this.ProductItem.Ingredients.Count)
            {
                throw new GameControllerCore.MisconfigurationException(
                    @$"Production component requires a list of inserters.
                       The number of inserters ({inserters.Count}) must match
                    the number of ingredients ({this.ProductItem.Ingredients.Count})
                    for the product {this.Product}
                    "
                );
            }
            // TODO: production manages the "enabled state" (new concept) of inserters
            for (int i = 0; i < inserters.Count; i++)
            {
                inserters[i].resourceType = this.ProductItem.Ingredients.Keys.ToList()[i];
            }
            this.inserters = inserters;
        }

        public void GetDesiredResources(ProductionQueueRequests resource = null)
        {
            // This is a recursive function that will build a list of all the resources
            // needed to craft the desired product. On the first call, the resource
            // parameter will be null. That resource parameter corresponds to the
            // product we want to craft. If the product has ingredients, we will call
            // this function again for each ingredient. If the ingredient has ingredients,
            // we will call this function again for each ingredient of the ingredient.
            // This will continue until we reach a product that has no ingredients.

            if (resource == null)
            {
                resource = new ProductionQueueRequests
                {
                    Item = this.ProductItem,
                    Quantity = ProductionComponentCore.InputBufferMultiplier,
                };
                this.Requests = new();
                this.Intermediates = new();
            }

            // If the item has ingredients, get the desired resources for each ingredient.
            if (resource.Item.Ingredients.Count != 0)
            {
                foreach (KeyValuePair<string, uint> ingredient in resource.Item.Ingredients)
                {
                    // Add the ingredient to the list of desired resources.
                    ProductionQueueRequests toAdd = new()
                    {
                        Item = this.gameContent.Items[ingredient.Key],
                        Quantity = ingredient.Value * resource.Quantity,
                    };
                    this.GetDesiredResources(toAdd);
                }
            }

            // Check if we already have some of the desired resources.
            uint existingResources = 0;
            if (this.resources != null && this.resources.Resources.ContainsKey(resource.Item.Name))
            {
                existingResources = this.resources.Resources[resource.Item.Name];
            }

            // If we don't have enough of the desired resources,
            // add them to the list of requests.
            bool haveEnough = existingResources > resource.Quantity;
            if (!haveEnough)
            {
                // Add the resource to the dictionary of requests.
                uint desiredResources = resource.Quantity - existingResources;
                if (!this.Requests.ContainsKey(resource.Item.Name))
                {
                    this.Requests[resource.Item.Name] = 0;
                }
                this.Requests[resource.Item.Name] += desiredResources;

                // If the resource is craftable, add it to the list of Intermediates.
                // Unless it is the final product.
                if (resource.Item.Ingredients.Count != 0 && resource.Item.Name != this.Product)
                {
                    if (!this.Intermediates.ContainsKey(resource.Item.Name))
                    {
                        this.Intermediates[resource.Item.Name] = 0;
                    }
                    this.Intermediates[resource.Item.Name] += desiredResources;
                }
            }
        }

        public void Produce()
        {
            if (this.outputBufferFull)
            {
                try
                {
                    this.resources.CreateResources(this.ProductItem.Name, 1);
                    this.battery.Energy -= ProductionComponentCore.PowerUsage;
                    this.outputBufferFull = false;
                }
                catch (ResourcesComponentCore.ResourceException) { }
                return; // TODO: output buffer is full and won't clear without intervention
            }

            // If we have already started a craft, continue it.
            if (this.currentCraftProgress > 0)
            {
                this.currentCraftProgress += 1;
                if (this.currentCraftProgress >= this.ProductItem.CraftTime)
                {
                    this.resources.CreateResources(this.ProductItem.Name, 1);
                    this.currentCraftProgress = 0;
                }
                this.battery.Energy -= ProductionComponentCore.PowerUsage;
                return;
            }

            // Try to craft the desired product.
            bool canCraft = true;
            foreach (KeyValuePair<string, uint> ingredient in this.ProductItem.Ingredients)
            {
                if (
                    !this.resources.Resources.ContainsKey(ingredient.Key)
                    || this.resources.Resources[ingredient.Key] < ingredient.Value
                )
                {
                    canCraft = false;
                    break;
                }
            }

            // If we can craft the product, do so.
            if (canCraft)
            {
                foreach (KeyValuePair<string, uint> ingredient in this.ProductItem.Ingredients)
                {
                    this.resources.ConsumeResources(ingredient.Key, ingredient.Value);
                }
                try
                {
                    if (this.ProductItem.CraftTime == 1)
                    {
                        this.resources.CreateResources(this.ProductItem.Name, 1);
                    }
                    else
                    {
                        this.currentCraftProgress += 1;
                    }
                }
                catch (ResourcesComponentCore.ResourceException)
                {
                    this.outputBufferFull = true;
                }
                this.battery.Energy -= ProductionComponentCore.PowerUsage;
            }
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
                { "wood", new Item("wood") },
                { "nails", new Item("nails") },
                {
                    "planks",
                    new Item("planks", ingredients: new Dictionary<string, uint> { { "wood", 5 } })
                },
                {
                    "wall",
                    new Item(
                        "wall",
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
                { "wood", new Item("wood") },
                { "iron", new Item("iron") },
                {
                    "nails",
                    new Item("nails", ingredients: new Dictionary<string, uint> { { "iron", 1 } })
                },
                {
                    "frame",
                    new Item("frame", ingredients: new Dictionary<string, uint> { { "iron", 16 } })
                },
                {
                    "planks",
                    new Item(
                        "planks",
                        ingredients: new Dictionary<string, uint> { { "wood", 5 }, { "nails", 5 } }
                    )
                },
                {
                    "wall",
                    new Item(
                        "wall",
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
                { "wood", new Item("wood") },
                {
                    "planks",
                    new Item(
                        "planks",
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
        public void TestGetBaseDesiredResources()
        {
            ResourcesComponentCore resources = new(new TestProductionGameContent(), 100, 100);
            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new()
            {
                new(battery, resources, "plants", 1),
                new(battery, resources, "nails", 1),
            };

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
                battery,
                inserters,
                "wall"
            );
            production.GetDesiredResources();

            Assert.Equal(50u, production.Requests["wood"]);
            Assert.Equal(10u, production.Requests["nails"]);
        }

        [Fact]
        public void TestGetDesiredResources()
        {
            ResourcesComponentCore resources = new(new TestProductionGameContent(), 100, 100);
            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new()
            {
                new(battery, resources, "plants", 1),
                new(battery, resources, "nails", 1),
            };

            ProductionComponentCore production = new(
                new TestProductionGameContent(), //
                resources,
                battery,
                inserters,
                "wall"
            );
            production.GetDesiredResources();

            Assert.Equal(50u, production.Requests["wood"]);
            Assert.Equal(10u, production.Requests["nails"]);
            Assert.Equal(10u, production.Requests["planks"]);
        }

        [Fact]
        public void TestGetDesiredResourcesWithIron()
        {
            ResourcesComponentCore resources = new(
                new TestProductionGameContentWithIron(),
                100,
                100
            );
            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new()
            {
                new(battery, resources, "plants", 1),
                new(battery, resources, "nails", 1),
                new(battery, resources, "frame", 1),
            };

            ProductionComponentCore production = new(
                new TestProductionGameContentWithIron(),
                resources,
                battery,
                inserters,
                "wall"
            );
            production.GetDesiredResources();

            Assert.Equal(40u, production.Requests["wood"]);
            Assert.Equal(8u, production.Requests["planks"]);
            Assert.Equal(48u, production.Requests["nails"]);
            Assert.Equal(40u, production.Requests["wood"]);
            Assert.Equal(80u, production.Requests["iron"]);
        }

        [Fact]
        public void TestGetDesiredResourcesWithIronOversupply()
        {
            ResourcesComponentCore resources = new(
                new TestProductionGameContentWithIron(),
                weightCapacity: 200,
                volumeCapacity: 200
            );
            resources.CreateResources("iron", 200);

            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new()
            {
                new(battery, resources, "plants", 1),
                new(battery, resources, "nails", 1),
                new(battery, resources, "frame", 1),
            };

            ProductionComponentCore production = new(
                new TestProductionGameContentWithIron(),
                resources,
                battery,
                inserters,
                "wall"
            );
            production.GetDesiredResources();

            Assert.Equal(40u, production.Requests["wood"]);
            Assert.Equal(8u, production.Requests["planks"]);
            Assert.Equal(48u, production.Requests["nails"]);
            Assert.Equal(40u, production.Requests["wood"]);
        }

        [Fact]
        public void TestIntermediatesAreOffset()
        {
            ResourcesComponentCore resources = new(
                new TestProductionGameContentWithIron(),
                weightCapacity: 400,
                volumeCapacity: 400
            );
            resources.CreateResources("iron", 200);
            resources.CreateResources("planks", 8);

            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new()
            {
                new(battery, resources, "plants", 1),
                new(battery, resources, "nails", 1),
                new(battery, resources, "frame", 1),
            };

            ProductionComponentCore production = new(
                new TestProductionGameContentWithIron(),
                resources,
                battery,
                inserters,
                "wall"
            );
            production.GetDesiredResources();

            Assert.Equal(0u, production.Requests.GetValueOrDefault("planks", 0u));
        }

        [Fact]
        public void TestAllIntermediates()
        {
            ResourcesComponentCore resources = new(
                new TestProductionGameContentWithIron(),
                weightCapacity: 400,
                volumeCapacity: 400
            );

            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new()
            {
                new(battery, resources, "plants", 1),
                new(battery, resources, "nails", 1),
                new(battery, resources, "frame", 1),
            };

            ProductionComponentCore production = new(
                new TestProductionGameContentWithIron(),
                resources,
                battery,
                inserters,
                "wall"
            );
            production.GetDesiredResources();

            Assert.Equal(6, production.Requests.Count);
            Assert.Equal(3, production.Intermediates.Count);
            Assert.Equal( //
                production.Requests["planks"],
                production.Intermediates["planks"]
            );
            Assert.Equal( //
                production.Requests["nails"],
                production.Intermediates["nails"]
            );
            Assert.Equal( //
                production.Requests["frame"],
                production.Intermediates["frame"]
            );
        }

        [Fact]
        public void TestGetDesiredResourcesWithOffsets()
        {
            ResourcesComponentCore resources = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            resources.CreateResources("wood", 25);

            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new()
            {
                new(battery, resources, "plants", 1),
                new(battery, resources, "nails", 1),
            };

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
                battery,
                inserters,
                "wall"
            );
            production.GetDesiredResources();

            Assert.Equal(25u, production.Requests["wood"]);
            Assert.Equal(10u, production.Requests["nails"]);
            Assert.Equal(10u, production.Requests["planks"]);
        }

        [Fact]
        public void TestGetDesiredResourcesWithOffsetsOversupply()
        {
            ResourcesComponentCore resources = new(
                new TestProductionGameContent(),
                weightCapacity: 200,
                volumeCapacity: 200
            );
            resources.CreateResources("wood", 200);

            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new()
            {
                new(battery, resources, "plants", 1),
                new(battery, resources, "nails", 1),
            };

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
                battery,
                inserters,
                "wall"
            );
            production.GetDesiredResources();

            Assert.Equal(0u, production.Requests.GetValueOrDefault("wood", 0u));
            Assert.Equal(10u, production.Requests["nails"]);
            Assert.Equal(10u, production.Requests["planks"]);
        }

        [Fact]
        public void TestSimpleProduction()
        {
            ResourcesComponentCore resources = new(
                new TestProductionGameContent(),
                weightCapacity: 500,
                volumeCapacity: 500
            );
            resources.CreateResources("wood", 5);

            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new()
            { //
                new(battery, resources, "wood", 1),
            };

            Assert.Equal(5u, resources.Resources["wood"]);
            Assert.Equal(0u, resources.Resources.GetValueOrDefault("planks", 0u));

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
                battery,
                inserters,
                "planks"
            );
            production.Produce();

            Assert.Equal(0u, resources.Resources["wood"]);
            Assert.Equal(1u, resources.Resources["planks"]);
        }

        [Fact]
        public void TestCraftsNailsWhenWoodAlreadyPresentAndNoIngredients()
        {
            ResourcesComponentCore resources = new(
                new TestProductionGameContent(),
                weightCapacity: 500,
                volumeCapacity: 500
            );
            resources.CreateResources("wood", 5);

            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new() { };

            Assert.Equal(5u, resources.Resources["wood"]);
            Assert.Equal(0u, resources.Resources.GetValueOrDefault("nails", 0u));

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
                battery,
                inserters,
                "nails"
            );
            production.Produce();

            Assert.Equal(5u, resources.Resources["wood"]);
            Assert.Equal(1u, resources.Resources["nails"]);
        }

        [Fact]
        public void TestOutputBuffer()
        {
            ResourcesComponentCore resources = new(
                new TestProductionGameContent(),
                weightCapacity: 500,
                volumeCapacity: 500
            );
            resources.CreateResources("wall", 4);

            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new()
            { //
                new(battery, resources, "wall", 1),
            };

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
                battery,
                inserters,
                "house"
            );
            production.Produce();
            Assert.True(production.outputBufferFull);
            Assert.Equal(0u, resources.Resources["house"]);
            Assert.Equal(0u, resources.Resources["wall"]);
        }

        [Fact]
        public void TestOutputBufferCanEmpty()
        {
            ResourcesComponentCore resources = new(
                new TestProductionGameContent(),
                weightCapacity: 1000,
                volumeCapacity: 1000
            );
            resources.CreateResources("wall", 8);

            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new() { new(battery, resources, "wall", 1) };

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
                battery,
                inserters,
                "house"
            );
            production.Produce();
            Assert.True(production.outputBufferFull);
            Assert.Equal(0u, resources.Resources["house"]);
            Assert.Equal(4u, resources.Resources["wall"]);

            // Toss the walls out so we can output a completed house
            resources.ConsumeResources("wall", 4);
            production.Produce();
            Assert.False(production.outputBufferFull);
            Assert.Equal(0u, resources.Resources["wall"]);
            Assert.Equal(1u, resources.Resources["house"]);
        }

        [Fact]
        public void TestCraftMultiple()
        {
            ResourcesComponentCore resources = new(
                new TestProductionGameContent(),
                weightCapacity: 500,
                volumeCapacity: 500
            );
            resources.CreateResources("wood", 20);

            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new()
            { //
                new(battery, resources, "wood", 1),
            };

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
                battery,
                inserters,
                "planks"
            );
            production.Produce();
            production.Produce();
            production.Produce();

            Assert.Equal(5u, resources.Resources["wood"]);
            Assert.Equal(3u, resources.Resources["planks"]);
        }

        [Fact]
        public void TestWithCraftTime()
        {
            ResourcesComponentCore resources = new(
                new TestProductionCraftTime(),
                weightCapacity: 500,
                volumeCapacity: 500
            );
            resources.CreateResources("wood", 5);

            BatteryComponentCore battery = new(100, 100);
            List<InserterComponentCore> inserters = new()
            { //
                new(battery, resources, "wood", 1),
            };

            ProductionComponentCore production = new(
                new TestProductionCraftTime(),
                resources,
                battery,
                inserters,
                "planks"
            );

            production.Produce();
            Assert.Equal(89u, battery.Energy);
            Assert.Equal(0u, resources.Resources["wood"]);
            Assert.Equal(0u, resources.Resources.GetValueOrDefault("planks", 0u));
            Assert.Equal(1u, production.currentCraftProgress);
            Assert.Equal("33%", production.PrecentProgressStatus);

            production.Produce();
            Assert.Equal(79u, battery.Energy);
            Assert.Equal(0u, resources.Resources["wood"]);
            Assert.Equal(0u, resources.Resources.GetValueOrDefault("planks", 0u));
            Assert.Equal(2u, production.currentCraftProgress);
            Assert.Equal("67%", production.PrecentProgressStatus);

            production.Produce();
            Assert.Equal(69u, battery.Energy);
            Assert.Equal(0u, resources.Resources["wood"]);
            Assert.Equal(1u, resources.Resources["planks"]);
        }
    }
}
