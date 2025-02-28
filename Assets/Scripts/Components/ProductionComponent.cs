namespace Assets.Scripts.Components.Core
{
    using System.Collections.Generic;
    using Assets.Scripts.Core;

    public class ProductionComponentCore
    {
        public GameContent.Item Product;
        public uint Quantity;
        public List<ProductionQueueRequests> Requests = new();
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
            string product,
            uint quantity
        )
        {
            this.gameContent = gameContent;
            this.Product = this.gameContent.Items[product];
            this.Quantity = quantity;
            this.resources = resources;
        }

        public void GetDesiredResouces(ProductionQueueRequests resource = null)
        {
            if (resource == null)
            {
                resource = new ProductionQueueRequests
                {
                    Item = this.Product,
                    Quantity = this.Quantity,
                };
                this.Requests = new();
            }

            if (resource.Item.Ingredients.Count != 0)
            {
                foreach (KeyValuePair<string, uint> ingredient in resource.Item.Ingredients)
                {
                    ProductionQueueRequests toAdd = new()
                    {
                        Item = this.gameContent.Items[ingredient.Key],
                        Quantity = ingredient.Value * resource.Quantity,
                    };
                    this.GetDesiredResouces(toAdd);
                }
            }

            uint existingResources = 0;
            if (this.resources != null && this.resources.Resources.ContainsKey(resource.Item.Name))
            {
                existingResources = this.resources.Resources[resource.Item.Name];
            }

            ProductionQueueRequests _resources = new()
            {
                Item = resource.Item,
                Quantity =
                    (existingResources < resource.Quantity)
                        ? resource.Quantity - existingResources
                        : 0,
            };

            this.Requests.Add(_resources);

            return;
        }

        public void Produce()
        {
            // TODO: craft time
            // TODO: craft ingredients
            // TODO: craft multiple at once

            if (this.Quantity == 0)
            {
                return;
            }

            if (this.outputBufferFull)
            {
                this.resources.CreateResources(this.Product.Name, 1);
                if (this.Quantity > 0)
                {
                    this.Quantity -= 1;
                }
                this.outputBufferFull = false;
                return;
            }

            // Try to craft the desired quantity of product.
            uint desiredQuantity = this.Quantity;
            for (uint i = 0; i < desiredQuantity; i++)
            {
                // Check if we can make the desired product directly.
                bool canCraft = true;
                foreach (KeyValuePair<string, uint> ingredient in this.Product.Ingredients)
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
                    foreach (KeyValuePair<string, uint> ingredient in this.Product.Ingredients)
                    {
                        this.resources.ConsumeResources(ingredient.Key, ingredient.Value);
                    }
                    try
                    {
                        this.resources.CreateResources(this.Product.Name, 1);
                        this.Quantity -= 1;
                    }
                    catch (ResourcesComponentCore.ResourceException exc)
                    {
                        this.outputBufferFull = true;
                    }
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
        public GameContent.Item Product => this.core.Product;
        public uint Quantity => this.core.Quantity;

        public void Instantiate(ResourcesComponent resources, string product, uint quantity)
        {
            this.core = new(new FactoryGameContent(), resources.core, product, quantity);
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
                new TestProductionGameContent(),
                null,
                "wall",
                4
            );
            production.GetDesiredResouces();

            Dictionary<string, uint> requests = new();
            foreach (
                ProductionComponentCore.ProductionQueueRequests resource in production.Requests
            )
            {
                if (!requests.ContainsKey(resource.Item.Name))
                {
                    requests[resource.Item.Name] = 0;
                }
                requests[resource.Item.Name] += resource.Quantity;
            }

            Assert.Equal(100u, requests["wood"]);
            Assert.Equal(20u, requests["nails"]);
        }

        [Fact]
        public void TestGetDesiredResouces()
        {
            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                null,
                "wall",
                4
            );
            production.GetDesiredResouces();

            Dictionary<string, uint> requests = new();
            foreach (
                ProductionComponentCore.ProductionQueueRequests resource in production.Requests
            )
            {
                if (!requests.ContainsKey(resource.Item.Name))
                {
                    requests[resource.Item.Name] = 0;
                }
                requests[resource.Item.Name] += resource.Quantity;
            }

            Assert.Equal(100u, requests["wood"]);
            Assert.Equal(20u, requests["nails"]);
            Assert.Equal(20u, requests["planks"]);
        }

        [Fact]
        public void TestGetDesiredResoucesWithIron()
        {
            ProductionComponentCore production = new(
                new TestProductionGameContentWithIron(),
                null,
                "wall",
                4
            );
            production.GetDesiredResouces();

            Dictionary<string, uint> requests = new();
            foreach (
                ProductionComponentCore.ProductionQueueRequests resource in production.Requests
            )
            {
                if (!requests.ContainsKey(resource.Item.Name))
                {
                    requests[resource.Item.Name] = 0;
                }
                requests[resource.Item.Name] += resource.Quantity;
            }

            Assert.Equal(80u, requests["wood"]);
            Assert.Equal(16u, requests["planks"]);
            Assert.Equal(96u, requests["nails"]);
            Assert.Equal(80u, requests["wood"]);
            Assert.Equal(160u, requests["iron"]);
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
                "wall",
                4
            );
            production.GetDesiredResouces();

            Dictionary<string, uint> requests = new();
            foreach (
                ProductionComponentCore.ProductionQueueRequests resource in production.Requests
            )
            {
                if (!requests.ContainsKey(resource.Item.Name))
                {
                    requests[resource.Item.Name] = 0;
                }
                requests[resource.Item.Name] += resource.Quantity;
            }

            Assert.Equal(80u, requests["wood"]);
            Assert.Equal(16u, requests["planks"]);
            Assert.Equal(96u, requests["nails"]);
            Assert.Equal(80u, requests["wood"]);
        }

        [Fact]
        public void TestGetDesiredResoucesWithOffsets()
        {
            ResourcesComponentCore resouces = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            resouces.CreateResources("wood", 50);

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resouces,
                "wall",
                4
            );
            production.GetDesiredResouces();

            Dictionary<string, uint> requests = new();
            foreach (
                ProductionComponentCore.ProductionQueueRequests resource in production.Requests
            )
            {
                if (!requests.ContainsKey(resource.Item.Name))
                {
                    requests[resource.Item.Name] = 0;
                }
                requests[resource.Item.Name] += resource.Quantity;
            }

            Assert.Equal(50u, requests["wood"]);
            Assert.Equal(20u, requests["nails"]);
            Assert.Equal(20u, requests["planks"]);
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
                "wall",
                4
            );
            production.GetDesiredResouces();

            Dictionary<string, uint> requests = new();
            foreach (
                ProductionComponentCore.ProductionQueueRequests resource in production.Requests
            )
            {
                if (!requests.ContainsKey(resource.Item.Name))
                {
                    requests[resource.Item.Name] = 0;
                }
                requests[resource.Item.Name] += resource.Quantity;
            }

            Assert.Equal(0u, requests["wood"]);
            Assert.Equal(20u, requests["nails"]);
            Assert.Equal(20u, requests["planks"]);
        }

        [Fact]
        public void TestSimpleProduction()
        {
            ResourcesComponentCore resources = new ResourcesComponentCore(
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
                "planks",
                1
            );
            production.Produce();

            Assert.Equal(0u, resources.Resources["wood"]);
            Assert.Equal(1u, resources.Resources["planks"]);
        }

        [Fact]
        public void TestCraftsNailsWhenWoodAlreadyPresent()
        {
            ResourcesComponentCore resources = new ResourcesComponentCore(
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
                "nails",
                1
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
                "house",
                1
            );
            production.Produce();
            Assert.True(production.outputBufferFull);
            Assert.Equal(0u, resources.Resources["house"]);
            Assert.Equal(0u, resources.Resources["wall"]);
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
                "planks",
                2
            );
            production.Produce();
            production.Produce();
            production.Produce(); // Should do nothing

            Assert.Equal(10u, resources.Resources["wood"]);
            Assert.Equal(2u, resources.Resources["planks"]);
        }

        [Fact]
        public void TestCraftMultipleAtOnce()
        {
            ResourcesComponentCore resources = new(
                new TestProductionGameContent(),
                weightCapacity: 4000,
                volumeCapacity: 4000
            );
            resources.CreateResources("wood", 20);

            ProductionComponentCore production = new(
                new TestProductionGameContent(),
                resources,
                "planks",
                2
            );
            production.Produce(); // Should make 2 at once

            Assert.Equal(0u, production.Quantity);
            Assert.Equal(10u, resources.Resources["wood"]);
            Assert.Equal(2u, resources.Resources["planks"]);
        }
    }
}
