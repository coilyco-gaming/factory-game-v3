using System.Collections.Generic;

namespace Assets.Scripts.Core
{
    public class GameContent
    {
        public virtual Dictionary<string, Object> Objects { get; }
        public virtual Dictionary<string, Item> Items { get; }

        public class Object { }

        public class Item
        {
            public string Name;
            public uint Weight;
            public uint Volume;
            public uint CraftTime;
            public uint StackSize;
            public Dictionary<string, uint> Ingredients;
            public bool spawnable;

            public Item(
                string name,
                uint weight = 1,
                uint volume = 1,
                uint craftTime = 1,
                uint stackSize = 1,
                Dictionary<string, uint> ingredients = null,
                bool spawnable = false
            )
            {
                this.Name = name;
                this.Weight = weight;
                this.Volume = volume;
                this.CraftTime = craftTime;
                this.StackSize = stackSize;
                this.Ingredients = ingredients ?? new();
                this.spawnable = spawnable;
            }
        }
    }
}
