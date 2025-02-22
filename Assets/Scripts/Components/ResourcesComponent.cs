using System;
using System.Collections.Generic;
using Assets.Scripts.WorldObjects;
using UnityEngine;

namespace Assets.Scripts.Components
{
    public class ResourcesComponent : MonoBehaviour
    {
        // FIELDS //

        public WorldObject worldObject;

        // PROPERTIES //

        public Dictionary<string, int> Resources { get; set; } = null;

        public int TotalResourceCapacity { get; set; }

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

        // CLASSES //

        public class ResourceException : Exception
        {
            public ResourceException(string message)
                : base(message) { }
        }

        // FUNCTIONS //

        public void Instantiate(
            WorldObject worldObject,
            int TotalResourceCapacity,
            Dictionary<string, int> ResourcesOnCreate = null
        )
        {
            this.worldObject = worldObject;
            this.TotalResourceCapacity = TotalResourceCapacity;
            this.Resources = ResourcesOnCreate ?? new();
        }

        public bool ConsumeResources(string resourceName, int amountToConsume)
        {
            int availableResources = this.Resources.GetValueOrDefault(resourceName, 0);
            if (availableResources < amountToConsume)
            {
                throw new ResourceException(
                    $"Not enough {amountToConsume} {resourceName} to consume"
                );
            }

            this.Resources[resourceName] -= amountToConsume;
            return true;
        }

        public bool GiveResources(ResourcesComponent target, string resourceName, int amountToGive)
        {
            int availableResources = this.Resources.GetValueOrDefault(resourceName, 0);
            if (availableResources < amountToGive)
            {
                throw new ResourceException(
                    $"Not enough {amountToGive} {resourceName} to give {target.worldObject.WorldObjectType}"
                );
            }

            if (target.RemainingResourceCapacity < amountToGive)
            {
                throw new ResourceException(
                    $"Not enough capacity to recieve {amountToGive} {resourceName}"
                );
            }

            this.Resources[resourceName] -= amountToGive;
            int currentResources = target.Resources.GetValueOrDefault(resourceName, 0);
            target.Resources[resourceName] = currentResources + amountToGive;
            return true;
        }

        public bool TakeResouces(ResourcesComponent target, string resourceName, int amountToTake)
        {
            return target.GiveResources(this, resourceName, amountToTake);
        }
    }
}
