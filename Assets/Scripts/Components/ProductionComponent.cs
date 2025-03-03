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

        private static uint PowerUsage = 10;
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

        public bool OutputStacksFullfilled =>
            this.resources.Resources.GetValueOrDefault(this.Product, 0u)
            >= this.ProductItem.StackSize;
        public bool OutputWeightFull =>
            this.ProductItem.Weight > this.resources.RemainingWeightCapacity;
        public bool OutputVolumeFull =>
            this.ProductItem.Volume > this.resources.RemainingVolumeCapacity;

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
                pair => pair.Value * this.ProductItem.StackSize
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
            for (int i = 0; i < inserters.Count; i++)
            {
                inserters[i].resourceType = this.ProductItem.Ingredients.Keys.ToList()[i];
            }
        }

        public void Produce()
        {
            // If we have already started a craft, continue it.
            if (this.currentCraftProgress > 0)
            {
                // Perform no crafting if the battery is empty.
                try
                {
                    this.battery.Energy -= ProductionComponentCore.PowerUsage;
                }
                catch (BatteryComponentCore.BatteryCapacityException)
                {
                    return;
                }
                this.currentCraftProgress += 1;
                if (this.currentCraftProgress >= this.ProductItem.CraftTime)
                {
                    this.resources.ForceCreateResources(this.ProductItem.Name, 1);
                    this.currentCraftProgress = 0;
                }
            }

            // If we have already produced the desired quantity, return.
            if (this.OutputStacksFullfilled)
            {
                return;
            }

            // If we would not have enough space to store the output, return.
            if (this.OutputWeightFull || this.OutputVolumeFull)
            {
                return;
            }

            // Check if we can craft the desired product.
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
                // Perform no crafting if the battery is empty.
                try
                {
                    this.battery.Energy -= ProductionComponentCore.PowerUsage;
                }
                catch (BatteryComponentCore.BatteryCapacityException)
                {
                    return;
                }

                foreach (KeyValuePair<string, uint> ingredient in this.ProductItem.Ingredients)
                {
                    this.resources.ConsumeResources(ingredient.Key, ingredient.Value);
                }
                if (this.ProductItem.CraftTime == 1)
                {
                    this.resources.ForceCreateResources(this.ProductItem.Name, 1);
                }
                else
                {
                    this.currentCraftProgress += 1;
                }
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
        public void TestOutputFilled()
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
            Assert.Equal(0u, resources.Resources.GetValueOrDefault("house", 0u));
            Assert.Equal(4u, resources.Resources["wall"]);
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
            Assert.Equal(90u, battery.Energy);
            Assert.Equal(0u, resources.Resources["wood"]);
            Assert.Equal(0u, resources.Resources.GetValueOrDefault("planks", 0u));
            Assert.Equal(1u, production.currentCraftProgress);
            Assert.Equal("33%", production.PrecentProgressStatus);

            production.Produce();
            Assert.Equal(80u, battery.Energy);
            Assert.Equal(0u, resources.Resources["wood"]);
            Assert.Equal(0u, resources.Resources.GetValueOrDefault("planks", 0u));
            Assert.Equal(2u, production.currentCraftProgress);
            Assert.Equal("67%", production.PrecentProgressStatus);

            production.Produce();
            Assert.Equal(70u, battery.Energy);
            Assert.Equal(0u, resources.Resources["wood"]);
            Assert.Equal(1u, resources.Resources["planks"]);
        }
    }
}
