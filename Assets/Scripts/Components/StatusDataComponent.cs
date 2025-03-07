namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;

    public class StatusDataComponentCore
    {
        // PROPERTIES //

        public Func<StatusData> Data { get; set; } = null;

        // CLASSES //

        public class StatusData
        {
            public string Name { get; set; } = null;
            public Dictionary<string, string> Resources { get; set; } = null;
            public Dictionary<string, string> Info { get; set; } = null;
            public List<Dictionary<int, string>> Alerts { get; set; } = null;
        }
    }
}
