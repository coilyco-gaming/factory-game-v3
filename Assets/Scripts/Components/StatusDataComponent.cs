using System;
using System.Collections.Generic;
using UnityEngine;

namespace Assets.Scripts.Components
{
    public class StatusDataComponent : MonoBehaviour
    {
        // PROPERTIES //

        public Func<StatusData> Data { get; set; } = null;

        // CLASSES //

        public class StatusData
        {
            public string Name { get; set; } = null;
            public string Objective { get; set; } = null;
            public Dictionary<string, string> Info { get; set; } = null;
            public List<Dictionary<int, string>> Alerts { get; set; } = null;
        }

        public void Instantiate() { }
    }
}
