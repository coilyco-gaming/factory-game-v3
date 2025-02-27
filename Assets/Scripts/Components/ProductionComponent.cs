using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Core;

namespace Assets.Scripts.Components.Core
{
    public class ProductionComponentCore
    {
        public GameContent.Item Product;
        public uint Quantity;
        private GameContent gameContent;

        public ProductionComponentCore(GameContent gameContent, string product, uint quantity)
        {
            this.gameContent = gameContent;
            this.Product = this.gameContent.Items[product];
            this.Quantity = quantity;
        }

        public Dictionary<string, uint> GetBaseResources()
        {
            // Hydrate the input dictionary with the game content.
            // The purpose of the bool becomes clear in the next step.
            List<Tuple<GameContent.Item, uint, bool>> _resources = this.GetBaseResources(
                new List<Tuple<GameContent.Item, uint, bool>>
                {
                    new(this.Product, this.Quantity, false),
                }
            );
            // Return to our original format on the way out.
            return _resources.ToDictionary(r => r.Item1.Name, r => r.Item2);
        }

        public List<Tuple<GameContent.Item, uint, bool>> GetBaseResources(
            List<Tuple<GameContent.Item, uint, bool>> resources
        )
        {
            // Iterate over the resources and check if they are base resources.
            // The boolean in the tuple is used to keep track of
            // whether the resource is a base resource.
            List<Tuple<GameContent.Item, uint, bool>> _resources = new();
            foreach (Tuple<GameContent.Item, uint, bool> resource in resources)
            {
                if (resource.Item1.Ingredients.Count != 0)
                {
                    // If the resource has ingredients, it is not a base resource.
                    // Add the ingredients to the list of resources.
                    foreach (KeyValuePair<string, uint> ingredient in resource.Item1.Ingredients)
                    {
                        _resources.Add(
                            new Tuple<GameContent.Item, uint, bool>(
                                this.gameContent.Items[ingredient.Key],
                                ingredient.Value * resource.Item2,
                                false // Assume by default that this isn't a base resource.
                            )
                        );
                    }
                }
                else
                {
                    // If the resource has no ingredients, it is a base resource.
                    // Add it to the list of resources.
                    _resources.Add(
                        new Tuple<GameContent.Item, uint, bool>(
                            resource.Item1,
                            resource.Item2,
                            true // Mark this as a base resource.
                        )
                    );
                }
            }

            // If there are no base resources in the list, return the list.
            if (_resources.All(r => r.Item3))
            {
                return _resources;
            }

            // If there are base resources in the list, call the function recursively.
            return this.GetBaseResources(_resources);
        }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Unity;
    using UnityEngine;

    public class ProductionComponent : MonoBehaviour
    {
        public ProductionComponentCore core;
        public GameContent.Item Product => this.core.Product;
        public uint Quantity => this.core.Quantity;

        public void Instantiate(string product, uint quantity)
        {
            this.core = new(new FactoryGameContent(), product, quantity);
        }

        public Dictionary<string, uint> GetBaseResources() => this.core.GetBaseResources();
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Xunit;

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
            };
    }

    public class ProductionComponentTest
    {
        [Fact]
        public void TestGetBaseResources()
        {
            ProductionComponentCore production = new(new TestProductionGameContent(), "wall", 2);
            Dictionary<string, uint> baseResources = production.GetBaseResources();
            Assert.Equal(50u, baseResources["wood"]);
            Assert.Equal(10u, baseResources["nails"]);
        }
    }
}
