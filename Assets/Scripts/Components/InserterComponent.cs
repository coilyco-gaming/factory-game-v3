// Inserter components query every nearby tile, once a tick,
// and insert matching items into the owner's resource inventory.


namespace Assets.Scripts.Components.Core
{
    using System.Collections.Generic;
    using Assets.Scripts.Core;

    public class InserterComponentCore
    {
        private ResourcesComponentCore resources;
        private string resourceType = "";
        private uint insertionRate = 0;

        public void Instantiate(
            ResourcesComponentCore resources = null,
            string resourceType = "",
            uint insertionRate = 0
        )
        {
            this.resources = resources ?? new ResourcesComponentCore(new GameContent());
            this.resourceType = resourceType;
            this.insertionRate = insertionRate;
        }

        public void Insert(List<ResourcesComponentCore> localResources)
        {
            foreach (
                ResourcesComponentCore localResource in localResources
                    ?? new List<ResourcesComponentCore>()
            )
            {
                try
                {
                    // TODO: pass in a flag to supress alerts
                    this.resources.TakeResouces(
                        localResource ?? new ResourcesComponentCore(new GameContent()),
                        this.resourceType,
                        this.insertionRate
                    );
                }
                catch (ResourcesComponentCore.ResourceException)
                {
                    continue;
                }
            }
        }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Assets.Scripts.Unity;
    using Assets.Scripts.WorldObjects.FactoryGame;
    using Assets.Scripts.WorldObjects.Unity;
    using UnityEngine;

    public class InserterComponent : MonoBehaviour
    {
        public readonly InserterComponentCore core = new();

        public void Instantiate(
            ResourcesComponent resources,
            string resourceType = "",
            uint insertionRate = 0
        ) => this.core.Instantiate(resources.core, resourceType, insertionRate);

        // TODO: this is WAY TO MUCH logic to be outside of unit tests
        // TODO: we need to create a GameControllerCore so we can test this function
        public void Insert(WorldObject worldObject, GameController gameController)
        {
            List<System.Numerics.Vector2> adjacentTiles = new()
            {
                new System.Numerics.Vector2( // Above
                    worldObject.GridPosition.X + 0,
                    worldObject.GridPosition.Y + 1
                ),
                new System.Numerics.Vector2( // Top Right
                    worldObject.GridPosition.X + 1,
                    worldObject.GridPosition.Y + 1
                ),
                new System.Numerics.Vector2( // Right
                    worldObject.GridPosition.X + 1,
                    worldObject.GridPosition.Y + 0
                ),
                new System.Numerics.Vector2( // Bottom Right
                    worldObject.GridPosition.X + 1,
                    worldObject.GridPosition.Y - 1
                ),
                new System.Numerics.Vector2( // Below
                    worldObject.GridPosition.X + 0,
                    worldObject.GridPosition.Y - 1
                ),
                new System.Numerics.Vector2( // Bottom Left
                    worldObject.GridPosition.X - 1,
                    worldObject.GridPosition.Y - 1
                ),
                new System.Numerics.Vector2( // Left
                    worldObject.GridPosition.X + -1,
                    worldObject.GridPosition.Y + 0
                ),
                new System.Numerics.Vector2( // Top Left
                    worldObject.GridPosition.X + -1,
                    worldObject.GridPosition.Y + 1
                ),
            };
            List<WorldObject> localWorldObjects = adjacentTiles
                .SelectMany(adjacentTile =>
                    gameController.GetWorldObjectsByPosition(adjacentTile)
                    ?? Enumerable.Empty<WorldObject>()
                )
                .ToList();
            List<ResourcesComponent> localResources = localWorldObjects
                .Select(localWorldObject => localWorldObject.Resources)
                .ToList();
            this.core.Insert(localResources.ConvertAll(localResource => localResource.core));
        }

        internal void Insert(
            WorldObjectFactory worldObjectFactory,
            GameController gameController
        ) => throw new NotImplementedException();
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class InserterComponentTest
    {
        [Fact]
        public void TestNulls()
        {
            InserterComponentCore inserter = new();
            inserter.Instantiate();
            inserter.Insert(null);
            inserter.Insert(new List<ResourcesComponentCore>());
            inserter.Insert(new List<ResourcesComponentCore>() { null });
        }

        [Fact]
        public void TestInsertCapacityOverflow()
        {
            ResourcesComponentCore resources = new(new TestGameContent());
            ResourcesComponentCore localResource = new(new TestGameContent());
            InserterComponentCore inserter = new();

            resources.Instantiate(1, 1, new Dictionary<string, uint> { { "wood", 1 } });
            localResource.Instantiate(1, 1, new Dictionary<string, uint> { { "wood", 1 } });

            inserter.Instantiate(resources, "wood", 1);
            inserter.Insert(new List<ResourcesComponentCore> { localResource });

            Assert.Equal(1u, resources.Resources["wood"]);
            Assert.Equal(1u, localResource.Resources["wood"]);
        }

        [Fact]
        public void TestInsert()
        {
            ResourcesComponentCore resources = new(new TestGameContent());
            ResourcesComponentCore localResource = new(new TestGameContent());
            InserterComponentCore inserter = new();

            resources.Instantiate(2, 2, new Dictionary<string, uint> { { "wood", 1 } });
            localResource.Instantiate(2, 2, new Dictionary<string, uint> { { "wood", 1 } });

            inserter.Instantiate(resources, "wood", 1);
            inserter.Insert(new List<ResourcesComponentCore> { localResource });

            Assert.Equal(2u, resources.Resources["wood"]);
            Assert.Equal(0u, localResource.Resources["wood"]);
        }

        [Fact]
        public void TestInsertMultiple()
        {
            ResourcesComponentCore resources = new(new TestGameContent());
            ResourcesComponentCore localResource1 = new(new TestGameContent());
            ResourcesComponentCore localResource2 = new(new TestGameContent());
            InserterComponentCore inserter = new();

            resources.Instantiate(3, 3, new Dictionary<string, uint> { { "wood", 1 } });
            localResource1.Instantiate(2, 2, new Dictionary<string, uint> { { "wood", 1 } });
            localResource2.Instantiate(2, 2, new Dictionary<string, uint> { { "wood", 1 } });

            inserter.Instantiate(resources, "wood", 1);
            inserter.Insert(new List<ResourcesComponentCore> { localResource1, localResource2 });

            Assert.Equal(3u, resources.Resources["wood"]);
            Assert.Equal(0u, localResource1.Resources["wood"]);
            Assert.Equal(0u, localResource2.Resources["wood"]);
        }
    }
}
