namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;

    public class ResourcesComponentCore
    {
        // PROPERTIES //

        public Dictionary<string, int> Resources { get; set; } = null;

        public int TotalResourceCapacity { get; set; } = 0;

        public virtual Dictionary<string, string> ResourceInfo
        {
            get
            {
                Dictionary<string, string> info = new();
                foreach (KeyValuePair<string, int> resource in this.Resources)
                {
                    info.Add(resource.Key, resource.Value.ToString());
                }
                return info;
            }
        }

        public int TotalResources
        {
            get
            {
                int total = 0;
                foreach (int resource in this.Resources.Values)
                {
                    total += resource;
                }
                return total;
            }
        }

        public int RemainingResourceCapacity
        {
            get
            {
                int capacity = this.TotalResourceCapacity;
                foreach (int resource in this.Resources.Values)
                {
                    capacity -= resource;
                }
                return capacity;
            }
        }

        public string LeastAvailableResource
        {
            get
            {
                string leastAvilable = null;
                int leastAvilableAmount = int.MaxValue;
                foreach (KeyValuePair<string, int> resource in this.Resources)
                {
                    if (resource.Value < leastAvilableAmount)
                    {
                        leastAvilable = resource.Key;
                        leastAvilableAmount = resource.Value;
                    }
                }
                return leastAvilable;
            }
        }

        public bool HasResources => this.TotalResources > 0;

        // CLASSES //

        public class ResourceException : Exception
        {
            public ResourceException(string message)
                : base(message) { }
        }

        public class ResourceCapacityException : ResourceException
        {
            public ResourceCapacityException(string message)
                : base(message) { }
        }

        public class ResourceQuantityException : ResourceException
        {
            public ResourceQuantityException(string message)
                : base(message) { }
        }

        // FUNCTIONS //

        public void Instantiate(
            int TotalResourceCapacity = 0,
            Dictionary<string, int> ResourcesOnCreate = null
        )
        {
            this.TotalResourceCapacity = TotalResourceCapacity;
            this.Resources = ResourcesOnCreate ?? new();
        }

        public bool ConsumeResources(string resourceName, int amountToConsume)
        {
            int availableResources = this.Resources.GetValueOrDefault(resourceName, 0);
            if (availableResources < amountToConsume)
            {
                throw new ResourceException(
                    $"Does not have {amountToConsume} {resourceName} to consume"
                );
            }

            this.Resources[resourceName] -= amountToConsume;
            return true;
        }

        public bool GiveResources(
            ResourcesComponentCore target,
            string resourceName,
            int amountToGive
        )
        {
            int availableResources = this.Resources.GetValueOrDefault(resourceName, 0);
            if (availableResources < amountToGive)
            {
                throw new ResourceQuantityException(
                    $"Does not have {amountToGive} {resourceName} to give"
                );
            }

            if (target.RemainingResourceCapacity < amountToGive)
            {
                throw new ResourceCapacityException(
                    $"Not enough capacity to recieve {amountToGive} {resourceName}"
                );
            }

            this.Resources[resourceName] -= amountToGive;
            int currentResources = target.Resources.GetValueOrDefault(resourceName, 0);
            target.Resources[resourceName] = currentResources + amountToGive;
            return true;
        }

        public bool TakeResouces(
            ResourcesComponentCore target,
            string resourceName,
            int amountToTake
        )
        {
            // TODO: catch the exceptions, emit them as alerts on the parent world object, then rethrow exception
            // TODO: give should alert on the giver, take should alert on the taker
            return target.GiveResources(this, resourceName, amountToTake);
        }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using UnityEngine;

    public class ResourcesComponent : MonoBehaviour
    {
        // FIELDS //

        private ResourcesComponentCore resourcesComponentCore = new();

        // PROPERTIES //

        // TODO: allow viewing these values in the unity inspector somehow
        public Dictionary<string, int> Resources
        {
            get => this.resourcesComponentCore.Resources;
            set => this.resourcesComponentCore.Resources = value;
        }

        public virtual Dictionary<string, string> ResourceInfo =>
            this.resourcesComponentCore.ResourceInfo;

        public int TotalResources => this.resourcesComponentCore.TotalResources;

        public int RemainingResourceCapacity =>
            this.resourcesComponentCore.RemainingResourceCapacity;

        public string LeastAvailableResource => this.resourcesComponentCore.LeastAvailableResource;

        public bool HasResources => this.resourcesComponentCore.HasResources;

        // FUNCTIONS //

        public void Instantiate(
            int TotalResourceCapacity = 0,
            Dictionary<string, int> ResourcesOnCreate = null
        ) => this.resourcesComponentCore.Instantiate(TotalResourceCapacity, ResourcesOnCreate);

        public bool ConsumeResources(string resourceName, int amountToConsume) =>
            this.resourcesComponentCore.ConsumeResources(resourceName, amountToConsume);

        public bool GiveResources(
            ResourcesComponent target,
            string resourceName,
            int amountToGive
        ) =>
            this.resourcesComponentCore.GiveResources(
                target.resourcesComponentCore,
                resourceName,
                amountToGive
            );

        public bool TakeResouces(
            ResourcesComponent target,
            string resourceName,
            int amountToTake
        ) =>
            this.resourcesComponentCore.TakeResouces(
                target.resourcesComponentCore,
                resourceName,
                amountToTake
            );
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class ResourcesComponentTest
    {
        [Fact]
        public void TestTrue()
        {
            ResourcesComponentCore resourcesComponent = new();
            resourcesComponent.Instantiate();
            Assert.True(true);
        }

        [Fact]
        public void TestFieldZeroStates()
        {
            ResourcesComponentCore resourcesComponent = new();
            resourcesComponent.Instantiate();
            Assert.Equal(0, resourcesComponent.TotalResources);
            Assert.Equal(0, resourcesComponent.RemainingResourceCapacity);
            Assert.Equal(resourcesComponent.Resources.Count, 0);
            Assert.Equal(resourcesComponent.ResourceInfo.Count, 0);
            Assert.Null(resourcesComponent.LeastAvailableResource);
            Assert.False(resourcesComponent.HasResources);
        }

        [Fact]
        public void TestResourcesOnInit()
        {
            ResourcesComponentCore resourcesComponent = new();
            resourcesComponent.Instantiate(
                TotalResourceCapacity: 100,
                ResourcesOnCreate: new()
                {
                    { "wood", 10 },
                    { "stone", 20 },
                    { "iron", 30 },
                }
            );
            Assert.Equal(60, resourcesComponent.TotalResources);
            Assert.Equal(40, resourcesComponent.RemainingResourceCapacity);
            Assert.Equal(resourcesComponent.Resources.Count, 3);
            Assert.Equal(resourcesComponent.ResourceInfo.Count, 3);
            Assert.Equal("wood", resourcesComponent.LeastAvailableResource);
            Assert.True(resourcesComponent.HasResources);
        }

        [Fact]
        public void TestConsumeResources()
        {
            ResourcesComponentCore resourcesComponent = new();
            resourcesComponent.Instantiate(
                TotalResourceCapacity: 100,
                ResourcesOnCreate: new()
                {
                    { "wood", 10 },
                    { "stone", 20 },
                    { "iron", 30 },
                }
            );
            resourcesComponent.ConsumeResources("wood", 5);
            Assert.Equal(55, resourcesComponent.TotalResources);
            Assert.Equal(45, resourcesComponent.RemainingResourceCapacity);
            Assert.Equal(5, resourcesComponent.Resources["wood"]);
        }

        [Fact]
        public void TestGiveAndTake()
        {
            ResourcesComponentCore resourcesComponent = new();
            resourcesComponent.Instantiate(
                TotalResourceCapacity: 100,
                ResourcesOnCreate: new()
                {
                    { "wood", 10 },
                    { "stone", 20 },
                    { "iron", 30 },
                }
            );

            ResourcesComponentCore targetResourcesComponent = new();
            targetResourcesComponent.Instantiate(
                TotalResourceCapacity: 100,
                ResourcesOnCreate: new()
                {
                    { "wood", 5 },
                    { "stone", 10 },
                    { "iron", 15 },
                }
            );

            resourcesComponent.GiveResources(targetResourcesComponent, "wood", 5);
            Assert.Equal(55, resourcesComponent.TotalResources);
            Assert.Equal(45, resourcesComponent.RemainingResourceCapacity);
            Assert.Equal(5, resourcesComponent.Resources["wood"]);
            Assert.Equal(35, targetResourcesComponent.TotalResources);
            Assert.Equal(65, targetResourcesComponent.RemainingResourceCapacity);
            Assert.Equal(10, targetResourcesComponent.Resources["wood"]);

            resourcesComponent.TakeResouces(targetResourcesComponent, "wood", 5);
            Assert.Equal(60, resourcesComponent.TotalResources);
            Assert.Equal(40, resourcesComponent.RemainingResourceCapacity);
            Assert.Equal(10, resourcesComponent.Resources["wood"]);
            Assert.Equal(30, targetResourcesComponent.TotalResources);
            Assert.Equal(70, targetResourcesComponent.RemainingResourceCapacity);
            Assert.Equal(5, targetResourcesComponent.Resources["wood"]);
        }

        [Fact]
        public void TestNotEnoughResourcesToGive()
        {
            ResourcesComponentCore resourcesComponent = new();
            resourcesComponent.Instantiate(
                TotalResourceCapacity: 100,
                ResourcesOnCreate: new()
                {
                    { "wood", 10 },
                    { "stone", 20 },
                    { "iron", 30 },
                }
            );

            ResourcesComponentCore targetResourcesComponent = new();
            targetResourcesComponent.Instantiate(
                TotalResourceCapacity: 100,
                ResourcesOnCreate: new()
                {
                    { "wood", 5 },
                    { "stone", 10 },
                    { "iron", 15 },
                }
            );

            Assert.Throws<ResourcesComponentCore.ResourceQuantityException>(
                () => resourcesComponent.GiveResources(targetResourcesComponent, "wood", 15)
            );
        }

        [Fact]
        public void TestNotEnoughCapacityToRecieve()
        {
            ResourcesComponentCore resourcesComponent = new();
            resourcesComponent.Instantiate(
                TotalResourceCapacity: 100,
                ResourcesOnCreate: new() { { "wood", 20 } }
            );

            ResourcesComponentCore targetResourcesComponent = new();
            targetResourcesComponent.Instantiate(
                TotalResourceCapacity: 100,
                ResourcesOnCreate: new() { { "wood", 90 } }
            );

            Assert.Throws<ResourcesComponentCore.ResourceCapacityException>(
                () => resourcesComponent.GiveResources(targetResourcesComponent, "wood", 20)
            );
        }
    }
}
