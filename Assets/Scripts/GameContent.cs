using System.Collections.Generic;

namespace Assets.Scripts.Core
{
    public class GameContent
    {
        public virtual Dictionary<string, Item> Items { get; }

        public class Item
        {
            public string Name;
            public uint Weight = 1;
            public uint Volume = 1;
            public uint CraftTime = 1;
            public uint StackSize = 1;
            public Dictionary<string, uint> Ingredients;
            public bool Spawnable = false; // Can the item be spawned in the world?
            public bool Manifests = false; // Can the object be created from nothing?

            public Item(
                string name,
                uint weight = 1,
                uint volume = 1,
                uint craftTime = 1,
                uint stackSize = 1,
                Dictionary<string, uint> ingredients = null,
                bool spawnable = false,
                bool manifests = false
            )
            {
                this.Name = name;
                this.Weight = weight;
                this.Volume = volume;
                this.CraftTime = craftTime;
                this.StackSize = stackSize;
                this.Ingredients = ingredients ?? new();
                this.Spawnable = spawnable;
                this.Manifests = manifests;
            }
        }
    }
}
