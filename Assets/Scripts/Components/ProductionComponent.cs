namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using Assets.Scripts.Core;

    public class ProductionComponentCore
    {
        public static uint InputBufferMultiplier = 2; // hold enough input buffer for 2 crafts
        public string Product;
        public GameContent.Item ProductItem => this.gameContent.Items[this.Product];
        public uint Quantity;
        public Dictionary<string, uint> Requests = new();
        public Dictionary<string, uint> Craftables = new();
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

        public class ProductionQueueRequests
        {
            public GameContent.Item Item;
            public uint CraftProgress;
            public uint Quantity;
        }

        public ProductionComponentCore(
            GameContent gameContent,
            ResourcesComponentCore resources,
            string product
        )
        {
            this.gameContent = gameContent;
            this.Product = product;
            this.resources = resources;
        }

        public void GetDesiredResouces(ProductionQueueRequests resource = null)
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
                this.Craftables = new();
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
                    this.GetDesiredResouces(toAdd);
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

                // If the resource is craftable, add it to the list of craftables.
                if (resource.Item.Ingredients.Count != 0)
                {
                    if (!this.Craftables.ContainsKey(resource.Item.Name))
                    {
                        this.Craftables[resource.Item.Name] = 0;
                    }
                    this.Craftables[resource.Item.Name] += desiredResources;
                }
            }
        }

        public void Produce()
        {
            // TODO: craft ingredients instead of just the end product

            if (this.outputBufferFull)
            {
                this.resources.CreateResources(this.ProductItem.Name, 1);
                this.outputBufferFull = false;
                return;
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
            }
        }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Assets.Scripts.Unity;
    using UnityEngine;

    public class ProductionComponent : MonoBehaviour
    {
        public ProductionComponentCore core;
        public GameContent.Item ProductItem => this.core.ProductItem;
        public uint Quantity => this.core.Quantity;

        public void Instantiate(ResourcesComponent resources, string product)
        {
            this.core = new(new FactoryGameContent(), resources.core, product);
        }
    }
}
#endif

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
        public void TestGetBaseDesiredResouces()
        {
            ProductionComponentCore production = new(
                new TestProductionGameContent(), //
                null,
                "wall"
            );
            production.GetDesiredResouces();

            Assert.Equal(50u, production.Requests["wood"]);
            Assert.Equal(10u, production.Requests["nails"]);
        }

        [Fact]
        public void TestGetDesiredResouces()
        {
            ProductionComponentCore production = new(
                new TestProductionGameContent(), //
                null,
                "wall"
            );
            production.GetDesiredResouces();

            Assert.Equal(50u, production.Requests["wood"]);
            Assert.Equal(10u, production.Requests["nails"]);
            Assert.Equal(10u, production.Requests["planks"]);
        }

        [Fact]
        public void TestGetDesiredResoucesWithIron()
        {
            ProductionComponentCore production = new(
                new TestProductionGameContentWithIron(),
                null,
                "wall"
            );
            production.GetDesiredResouces();

            Assert.Equal(40u, production.Requests["wood"]);
            Assert.Equal(8u, production.Requests["planks"]);
            Assert.Equal(48u, production.Requests["nails"]);
            Assert.Equal(40u, production.Requests["wood"]);
            Assert.Equal(80u, production.Requests["iron"]);
        }

        [Fact]
        public void TestGetDesiredResoucesWithIronOversupply()
        {
            ResourcesComponentCore resources = new(
                new TestProductionGameContentWithIron(),
                weightCapacity: 200,
                volumeCapacity: 200
            );
            resources.CreateResources("iron", 200);

            ProductionComponentCore production = new(
                new TestProductionGameContentWithIron(),
                resources,
                "wall"
            );
            production.GetDesiredResouces();

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

            ProductionComponentCore production = new(
                new TestProductionGameContentWithIron(),
                resources,
                "wall"
            );
            production.GetDesiredResouces();

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

            ProductionComponentCore production = new(
                new TestProductionGameContentWithIron(),
                resources,
                "wall"
            );
            production.GetDesiredResouces();

            Assert.Equal(6, production.Requests.Count);
            Assert.Equal(4, production.Craftables.Count);
            Assert.Equal( //
                production.Requests["wall"],
                production.Craftables["wall"]
            );
            Assert.Equal( //
                production.Requests["planks"],
                production.Craftables["planks"]
            );
            Assert.Equal( //
                production.Requests["nails"],
                production.Craftables["nails"]
            );
            Assert.Equal( //
                production.Requests["frame"],
                production.Craftables["frame"]
            );
        }

        [Fact]
        public void TestGetDesiredResoucesWithOffsets()
        {
            ResourcesComponentCore resouces = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            resouces.CreateResources("wood", 25);

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resouces,
                "wall"
            );
            production.GetDesiredResouces();

            Assert.Equal(25u, production.Requests["wood"]);
            Assert.Equal(10u, production.Requests["nails"]);
            Assert.Equal(10u, production.Requests["planks"]);
        }

        [Fact]
        public void TestGetDesiredResoucesWithOffsetsOversupply()
        {
            ResourcesComponentCore resouces = new(
                new TestResourcesGameContent(),
                weightCapacity: 200,
                volumeCapacity: 200
            );
            resouces.CreateResources("wood", 200);

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resouces,
                "wall"
            );
            production.GetDesiredResouces();

            Assert.Equal(0u, production.Requests.GetValueOrDefault("wood", 0u));
            Assert.Equal(10u, production.Requests["nails"]);
            Assert.Equal(10u, production.Requests["planks"]);
        }

        [Fact]
        public void TestSimpleProduction()
        {
            ResourcesComponentCore resources = new(
                new TestResourcesGameContent(),
                weightCapacity: 500,
                volumeCapacity: 500
            );
            resources.CreateResources("wood", 5);

            Assert.Equal(5u, resources.Resources["wood"]);
            Assert.Equal(0u, resources.Resources.GetValueOrDefault("planks", 0u));

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
                "planks"
            );
            production.Produce();

            Assert.Equal(0u, resources.Resources["wood"]);
            Assert.Equal(1u, resources.Resources["planks"]);
        }

        [Fact]
        public void TestCraftsNailsWhenWoodAlreadyPresent()
        {
            ResourcesComponentCore resources = new(
                new TestResourcesGameContent(),
                weightCapacity: 500,
                volumeCapacity: 500
            );
            resources.CreateResources("wood", 5);

            Assert.Equal(5u, resources.Resources["wood"]);
            Assert.Equal(0u, resources.Resources.GetValueOrDefault("nails", 0u));

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
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
            Assert.Equal(4u, resources.Resources["wall"]);

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
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

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
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

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
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

            ProductionComponentCore production = new(
                new TestProductionCraftTime(),
                resources,
                "planks"
            );

            production.Produce();
            Assert.Equal(0u, resources.Resources["wood"]);
            Assert.Equal(0u, resources.Resources.GetValueOrDefault("planks", 0u));
            Assert.Equal(1u, production.currentCraftProgress);
            Assert.Equal("33%", production.PrecentProgressStatus);

            production.Produce();
            Assert.Equal(0u, resources.Resources["wood"]);
            Assert.Equal(0u, resources.Resources.GetValueOrDefault("planks", 0u));
            Assert.Equal(2u, production.currentCraftProgress);
            Assert.Equal("67%", production.PrecentProgressStatus);

            production.Produce();
            Assert.Equal(0u, resources.Resources["wood"]);
            Assert.Equal(1u, resources.Resources["planks"]);
        }
    }
}
