using System.Collections.Generic;

namespace Assets.Scripts.Core
{
    public class GameContent
    {
        public virtual Dictionary<string, Item> Items { get; }

        public class Item
        {
            public string Name;
            public uint Weight;
            public uint Volume;
            public uint CraftTime;
            public Dictionary<string, uint> Ingredients;

            public Item(
                string name = "",
                uint weight = 1,
                uint volume = 1,
                uint craftTime = 1,
                Dictionary<string, uint> ingredients = null
            )
            {
                this.Name = name;
                this.Weight = weight;
                this.Volume = volume;
                this.CraftTime = craftTime;
                this.Ingredients = ingredients ?? new();
            }
        }
    }
}
