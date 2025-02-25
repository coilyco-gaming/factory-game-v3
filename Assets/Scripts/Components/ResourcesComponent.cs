namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;

    public class ResourcesComponentCore
    {
        // PROPERTIES //

        // TODO: reserve storage for certain kinds of resources

        public Dictionary<string, uint> Resources { get; private set; } = new();

        public uint TotalResourceCapacity { get; set; } = 0;

        public virtual Dictionary<string, string> ResourceInfo
        {
            get
            {
                Dictionary<string, string> info = new();
                foreach (KeyValuePair<string, uint> resource in this.Resources)
                {
                    info.Add(resource.Key, resource.Value.ToString());
                }
                return info;
            }
        }

        public uint TotalResources
        {
            get
            {
                uint total = 0;
                foreach (uint resource in this.Resources.Values)
                {
                    total += resource;
                }
                return total;
            }
        }

        public string LeastAvailableResource
        {
            get
            {
                string leastAvilable = null;
                uint leastAvilableAmount = int.MaxValue;
                foreach (KeyValuePair<string, uint> resource in this.Resources)
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

        public uint RemainingResourceCapacity
        {
            get
            {
                uint capacity = this.TotalResourceCapacity;
                foreach (uint resource in this.Resources.Values)
                {
                    capacity -= resource;
                }
                return capacity;
            }
        }

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
            uint TotalResourceCapacity = 0,
            Dictionary<string, uint> ResourcesOnCreate = null
        )
        {
            this.TotalResourceCapacity = TotalResourceCapacity;
            this.Resources = ResourcesOnCreate ?? new();
        }

        public bool ConsumeResources(string resourceName, uint amountToConsume)
        {
            uint availableResources = this.Resources.GetValueOrDefault(resourceName, (uint)0);
            if (availableResources < amountToConsume)
            {
                throw new ResourceException(
                    $"Does not have {amountToConsume} {resourceName} to consume"
                );
            }

            if (amountToConsume != 0)
            {
                this.Resources[resourceName] -= amountToConsume;
            }
            return true;
        }

        public bool GiveResources(
            ResourcesComponentCore target,
            string resourceName,
            uint amountToGive
        )
        {
            uint availableResources = this.Resources.GetValueOrDefault(resourceName, (uint)0);
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
            uint currentResources = target.Resources.GetValueOrDefault(resourceName, (uint)0);
            target.Resources[resourceName] = currentResources + amountToGive;
            return true;
        }

        public bool TakeResouces(
            ResourcesComponentCore target,
            string resourceName,
            uint amountToTake
        )
        {
            // TODO: catch the exceptions, emit them as alerts on both world objects, then rethrow exception
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

        public ResourcesComponentCore core = new();

        // PROPERTIES //

        // TODO: allow viewing these values in the unity inspector somehow
        public Dictionary<string, uint> Resources => this.core.Resources;

        public virtual Dictionary<string, string> ResourceInfo => this.core.ResourceInfo;

        public uint TotalResources => this.core.TotalResources;

        public uint RemainingResourceCapacity => this.core.RemainingResourceCapacity;

        public string LeastAvailableResource => this.core.LeastAvailableResource;

        public bool HasResources => this.core.HasResources;

        // FUNCTIONS //

        public void Instantiate(
            uint TotalResourceCapacity = 0,
            Dictionary<string, uint> ResourcesOnCreate = null
        ) => this.core.Instantiate(TotalResourceCapacity, ResourcesOnCreate);

        public bool ConsumeResources(string resourceName, uint amountToConsume) =>
            this.core.ConsumeResources(resourceName, amountToConsume);

        public bool GiveResources(
            ResourcesComponent target,
            string resourceName,
            uint amountToGive
        ) => this.core.GiveResources(target.core, resourceName, amountToGive);

        public bool TakeResouces(
            ResourcesComponent target,
            string resourceName,
            uint amountToTake
        ) => this.core.TakeResouces(target.core, resourceName, amountToTake);
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
            Assert.Equal((uint)0, resourcesComponent.TotalResources);
            Assert.Equal((uint)0, resourcesComponent.RemainingResourceCapacity);
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
            Assert.Equal((uint)60, resourcesComponent.TotalResources);
            Assert.Equal((uint)40, resourcesComponent.RemainingResourceCapacity);
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
            Assert.Equal((uint)55, resourcesComponent.TotalResources);
            Assert.Equal((uint)45, resourcesComponent.RemainingResourceCapacity);
            Assert.Equal((uint)5, resourcesComponent.Resources["wood"]);
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
            Assert.Equal((uint)55, resourcesComponent.TotalResources);
            Assert.Equal((uint)45, resourcesComponent.RemainingResourceCapacity);
            Assert.Equal((uint)5, resourcesComponent.Resources["wood"]);
            Assert.Equal((uint)35, targetResourcesComponent.TotalResources);
            Assert.Equal((uint)65, targetResourcesComponent.RemainingResourceCapacity);
            Assert.Equal((uint)10, targetResourcesComponent.Resources["wood"]);

            resourcesComponent.TakeResouces(targetResourcesComponent, "wood", 5);
            Assert.Equal((uint)60, resourcesComponent.TotalResources);
            Assert.Equal((uint)40, resourcesComponent.RemainingResourceCapacity);
            Assert.Equal((uint)10, resourcesComponent.Resources["wood"]);
            Assert.Equal((uint)30, targetResourcesComponent.TotalResources);
            Assert.Equal((uint)70, targetResourcesComponent.RemainingResourceCapacity);
            Assert.Equal((uint)5, targetResourcesComponent.Resources["wood"]);
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
