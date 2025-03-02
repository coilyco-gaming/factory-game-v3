// Inserter components query every nearby tile, once a tick,
// and insert matching items into the owner's resource inventory.


namespace Assets.Scripts.Components.Core
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Core;
    using Assets.Scripts.WorldObjects.Core;

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
            this.resources = resources ?? new ResourcesComponentCore(new GameContent(), 0, 0);
            this.resourceType = resourceType;
            this.insertionRate = insertionRate;
        }

        public void Insert(WorldObjectCore worldObject, GameControllerCore gameController)
        {
            // TODO: inserters consume power
            List<WorldObjectCore> localWorldObjects = gameController.GetAdjacentWorldObjects(
                worldObject.GridPosition
            );
            List<ResourcesComponentCore> localResources = localWorldObjects
                .Select(localWorldObject => localWorldObject.Resources)
                .ToList();

            foreach (
                ResourcesComponentCore localResource in localResources
                    ?? new List<ResourcesComponentCore>()
            )
            {
                try
                {
                    // TODO: pass in a flag to supress alerts
                    this.resources?.TakeResouces(
                        localResource ?? new ResourcesComponentCore(new GameContent(), 0, 0),
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

// namespace Assets.Scripts.Components.Tests
// {
//     using System.Collections.Generic;
//     using Assets.Scripts.Components.Core;
//     using Xunit;

//     public class InserterComponentTest
//     {
//         [Fact]
//         public void TestNulls()
//         {
//             InserterComponentCore inserter = new();
//             inserter.Instantiate();
//             inserter.Insert(null);
//             inserter.Insert(new List<ResourcesComponentCore>());
//             inserter.Insert(new List<ResourcesComponentCore>() { null });
//             inserter.Insert(
//                 new List<ResourcesComponentCore>()
//                 {
//                     new(null, weightCapacity: 0, volumeCapacity: 0),
//                 }
//             );
//             inserter.Insert(
//                 new List<ResourcesComponentCore>()
//                 {
//                     new(new TestResourcesGameContent(), weightCapacity: 100, volumeCapacity: 100),
//                 }
//             );
//         }

//         [Fact]
//         public void TestInsertCapacityOverflow()
//         {
//             ResourcesComponentCore resources = new(new TestResourcesGameContent(), 1, 1);
//             resources.CreateResources("wood", 1);
//             ResourcesComponentCore localResource = new(new TestResourcesGameContent(), 1, 1);
//             localResource.CreateResources("wood", 1);
//             InserterComponentCore inserter = new();

//             inserter.Instantiate(resources, "wood", 1);
//             inserter.Insert(new List<ResourcesComponentCore> { localResource });

//             Assert.Equal(1u, resources.Resources["wood"]);
//             Assert.Equal(1u, localResource.Resources["wood"]);
//         }

//         [Fact]
//         public void TestInsert()
//         {
//             ResourcesComponentCore resources = new(new TestResourcesGameContent(), 2, 2);
//             resources.CreateResources("wood", 1);
//             ResourcesComponentCore localResource = new(new TestResourcesGameContent(), 2, 2);
//             localResource.CreateResources("wood", 1);
//             InserterComponentCore inserter = new();

//             inserter.Instantiate(resources, "wood", 1);
//             inserter.Insert(new List<ResourcesComponentCore> { localResource });

//             Assert.Equal(2u, resources.Resources["wood"]);
//             Assert.Equal(0u, localResource.Resources["wood"]);
//         }

//         [Fact]
//         public void TestInsertMultiple()
//         {
//             ResourcesComponentCore resources = new(new TestResourcesGameContent(), 3, 3);
//             resources.CreateResources("wood", 1);
//             ResourcesComponentCore localResource1 = new(new TestResourcesGameContent(), 2, 2);
//             localResource1.CreateResources("wood", 1);
//             ResourcesComponentCore localResource2 = new(new TestResourcesGameContent(), 2, 2);
//             localResource2.CreateResources("wood", 1);
//             InserterComponentCore inserter = new();

//             inserter.Instantiate(resources, "wood", 1);
//             inserter.Insert(new List<ResourcesComponentCore> { localResource1, localResource2 });

//             Assert.Equal(3u, resources.Resources["wood"]);
//             Assert.Equal(0u, localResource1.Resources["wood"]);
//             Assert.Equal(0u, localResource2.Resources["wood"]);
//         }

//         [Fact]
//         public void TestEmptyGameContent()
//         {
//             InserterComponentCore inserter = new();
//             inserter.Instantiate();
//             inserter.Insert(new List<ResourcesComponentCore>());
//         }
//     }
// }
